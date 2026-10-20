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

// Other sections belong to the future planner; only message settings are consumed here.
#[derive(Debug, Deserialize)]
struct ConfigFile {
    schema_version: u32,
    #[serde(default)]
    message: MessageConfig,
}

#[derive(Debug, Default)]
pub struct ResolvedMessageConfig {
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
        .context("无法运行 git rev-parse")?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.contains("not a git repository") {
            return Ok(None);
        }
        bail!("Git 目录发现失败: {}", error.trim());
    }
    let path = String::from_utf8(output.stdout).context("Git 目录不是 UTF-8 路径")?;
    Ok(Some(PathBuf::from(path.trim_end_matches(['\r', '\n']))))
}

pub fn load(cwd: &Path, global: Option<&Path>) -> Result<ResolvedMessageConfig> {
    let mut result = ResolvedMessageConfig::default();
    if let Some(directory) = global {
        overlay(&mut result, &directory.join("config.toml"))?;
    }
    if let Some(directory) = git_path(cwd, "--git-common-dir")? {
        overlay(&mut result, &directory.join("rls/config.toml"))?;
    }
    Ok(result)
}

fn overlay(result: &mut ResolvedMessageConfig, path: &Path) -> Result<()> {
    let source = match std::fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).with_context(|| format!("读取配置 {}", path.display())),
    };
    let config: ConfigFile =
        toml::from_str(&source).with_context(|| format!("解析配置 {}", path.display()))?;
    if config.schema_version != 1 {
        bail!(
            "配置 {} 不支持 schema_version {}",
            path.display(),
            config.schema_version
        );
    }
    let message_version = config.message.version.unwrap_or(1);
    if message_version != 1 {
        bail!(
            "配置 {} 不支持 message.version {}，当前仅支持 1",
            path.display(),
            message_version
        );
    }
    if let Some(template) = config.message.template {
        if template.as_os_str().is_empty() {
            bail!("配置 {} 的 message.template 不能为空", path.display());
        }
        result.template = Some(path.parent().expect("config parent").join(template));
    }
    result.variables.extend(config.message.variables);
    Ok(())
}
