use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct MessageConfig {
    // Missing version denotes the original v1 format, independently in each layer.
    version: Option<u32>,
    template: Option<PathBuf>,
    #[serde(default)]
    variables: BTreeMap<String, toml::Value>,
}

#[derive(Debug, Deserialize)]
struct ConfigFile {
    schema_version: u32,
    #[serde(default)]
    release: ReleaseConfig,
    #[serde(default)]
    message: MessageConfig,
    #[serde(default)]
    branches: BTreeMap<String, BranchRule>,
}

#[derive(Debug, Deserialize)]
pub struct ReleaseConfig {
    #[serde(default = "default_remote")]
    pub remote: String,
}
fn default_remote() -> String {
    "origin".into()
}
impl Default for ReleaseConfig {
    fn default() -> Self {
        Self {
            remote: default_remote(),
        }
    }
}
#[derive(Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Increment {
    #[default]
    Counter,
    Manual,
    Script,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchRule {
    pub version_prefix: Option<String>,
    pub tag_prefix: Option<String>,
    pub tag_pattern: Option<String>,
    pub script: Option<ScriptConfig>,
    #[serde(default)]
    pub exclude_tags: Vec<String>,
    #[serde(default)]
    pub increment: Increment,
}

#[derive(Debug, Default)]
pub struct ResolvedMessageConfig {
    pub release: ReleaseConfig,
    pub branches: BTreeMap<String, BranchRule>,
    pub template: Option<PathBuf>,
    pub variables: BTreeMap<String, toml::Value>,
}

pub fn global_dir() -> Option<PathBuf> {
    std::env::var_os("RLS_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| directories::BaseDirs::new().map(|dirs| dirs.config_dir().join("rls")))
}

/// Returns None only when Git explicitly reports that cwd is outside a repository.
pub fn git_path(cwd: &Path, option: &str) -> Result<Option<PathBuf>> {
    let output = Command::new("git")
        .current_dir(cwd)
        .env("LC_ALL", "C")
        .args(["rev-parse", "--path-format=absolute", option])
        .output()
        .context(crate::tr!("无法运行 git rev-parse"))?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("not a git repository") {
            return Ok(None);
        }
        bail!("{}", crate::tr!("Git 目录发现失败: {}", error.trim()));
    }
    let path = String::from_utf8(output.stdout).context(crate::tr!("Git 目录不是 UTF-8 路径"))?;
    Ok(Some(PathBuf::from(path.trim_end_matches(['\r', '\n']))))
}

pub fn load(cwd: &Path, global: Option<&Path>) -> Result<ResolvedMessageConfig> {
    let mut result = ResolvedMessageConfig::default();
    if let Some(directory) = global {
        overlay(&mut result, &directory.join("config.toml"), false)?;
    }
    if let Some(directory) = git_path(cwd, "--git-common-dir")? {
        overlay(&mut result, &directory.join("rls/config.toml"), true)?;
    }
    Ok(result)
}

fn overlay(result: &mut ResolvedMessageConfig, path: &Path, repository: bool) -> Result<()> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| crate::tr!("读取配置 {}", path.display())),
    };
    let config: ConfigFile =
        toml::from_str(&source).with_context(|| crate::tr!("解析配置 {}", path.display()))?;
    if config.schema_version != 1 {
        bail!(
            "{}",
            crate::tr!(
                "配置 {} 不支持 schema_version {}",
                path.display(),
                config.schema_version
            )
        );
    }
    let message_version = config.message.version.unwrap_or(1);
    if message_version != 1 {
        bail!(
            "{}",
            crate::tr!(
                "配置 {} 不支持 message.version {}，当前仅支持 1",
                path.display(),
                message_version
            )
        );
    }
    if let Some(template) = config.message.template {
        if template.as_os_str().is_empty() {
            bail!(
                "{}",
                crate::tr!("配置 {} 的 message.template 不能为空", path.display())
            );
        }
        result.template = Some(path.parent().expect("config parent").join(template));
    }
    result.variables.extend(config.message.variables);
    if repository {
        for (name, rule) in &config.branches {
            rule.validate()
                .with_context(|| format!("{}: branches.{name}", path.display()))?;
        }
        result.branches = config.branches;
        result.release = config.release;
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptConfig {
    pub command: Vec<String>,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}
fn default_timeout() -> u64 {
    10
}

impl BranchRule {
    pub fn prefix(&self) -> &str {
        self.tag_prefix.as_deref().unwrap_or("")
    }
    fn validate(&self) -> Result<()> {
        use crate::i18n::text as l;
        if let Some(pattern) = &self.tag_pattern {
            if pattern.is_empty() || pattern.contains(['[', ']', '\\']) {
                bail!(
                    "{}",
                    l(
                        "tag_pattern 必须非空且只支持 * 和 ? 通配符",
                        "tag_pattern must be nonempty and supports only * and ? wildcards"
                    )
                );
            }
            if self.increment == Increment::Counter || self.version_prefix.is_some() {
                bail!(
                    "{}",
                    l(
                        "tag_pattern 不能与 counter 或 version_prefix 同时使用",
                        "tag_pattern cannot be combined with counter or version_prefix"
                    )
                );
            }
        } else {
            if self.increment == Increment::Script {
                bail!(
                    "{}",
                    l(
                        "script 模式需要 tag_pattern",
                        "script mode requires tag_pattern"
                    )
                );
            }
            let version = self
                .version_prefix
                .as_deref()
                .context(l("需要 version_prefix", "version_prefix is required"))?;
            if self.tag_prefix.is_none()
                || version
                    .split('.')
                    .any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()))
            {
                bail!(
                    "{}",
                    l(
                        "数字模式需要 tag_prefix 和点分数字 version_prefix",
                        "numeric mode requires tag_prefix and a dot-separated numeric version_prefix"
                    )
                );
            }
        }
        match (&self.increment, &self.script) {
            (Increment::Script, Some(script))
                if script.command.is_empty()
                    || script.command[0].trim().is_empty()
                    || script.command.iter().any(|s| s.contains('\0'))
                    || script.timeout_seconds == 0 =>
            {
                bail!(
                    "{}",
                    l(
                        "script.command 或 timeout_seconds 无效",
                        "invalid script.command or timeout_seconds"
                    )
                );
            }
            (Increment::Script, Some(_)) => {}
            (Increment::Script, None) => bail!(
                "{}",
                l(
                    "script 模式需要 script.command",
                    "script mode requires script.command"
                )
            ),
            (_, Some(_)) => bail!(
                "{}",
                l(
                    "只有 script 模式允许 script 配置",
                    "script configuration requires script mode"
                )
            ),
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mode_combinations_are_explicit() {
        let cases = [
            ("version_prefix='1.0'\ntag_prefix='v'", true),
            ("increment='manual'\ntag_pattern='r-*'", true),
            (
                "increment='script'\ntag_pattern='r-*'\n[script]\ncommand=['bash','./name.sh']",
                true,
            ),
            ("increment='counter'\ntag_pattern='r-*'", false),
            (
                "increment='manual'\ntag_pattern='r-*'\nversion_prefix='1.0'",
                false,
            ),
            (
                "increment='script'\nversion_prefix='1.0'\ntag_prefix='v'",
                false,
            ),
            ("increment='script'\ntag_pattern='r-*'", false),
            (
                "increment='script'\ntag_pattern='r-*'\n[script]\ncommand=[]",
                false,
            ),
            (
                "increment='script'\ntag_pattern='r-*'\n[script]\ncommand=['sh']\ntimeout_seconds=0",
                false,
            ),
            (
                "increment='manual'\ntag_pattern='r-*'\n[script]\ncommand=['sh']",
                false,
            ),
            ("increment='manual'\ntag_pattern='r-[ab]'", false),
            ("increment='manual'\ntag_pattern=''", false),
            (
                "increment='manual'\ntag_pattern='r-*'\ntag_order='ancestry'",
                false,
            ),
        ];
        for (source, valid) in cases {
            let result = toml::from_str::<BranchRule>(source)
                .map_err(anyhow::Error::from)
                .and_then(|rule| rule.validate());
            assert_eq!(result.is_ok(), valid, "{source}: {result:?}");
        }
    }
}
