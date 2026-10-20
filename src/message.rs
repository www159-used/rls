use crate::config::ResolvedMessageConfig;
use anyhow::{Context, Result, bail};
use minijinja::{Environment, UndefinedBehavior, context};
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_TEMPLATE: &str = "{% for release in releases %}{{ release.tag }}\nchangelog:\n{% for commit in release.commits %}{{ commit.subject }}\n{% endfor %}{% if not loop.last %}\n\n{% endif %}{% endfor %}";

#[derive(Debug, Deserialize, Serialize)]
pub struct ReleaseRecord {
    pub schema_version: u32,
    pub release_id: String,
    pub releases: Vec<Release>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub status: ReleaseStatus,
    pub tag: String,
    pub sha: String,
    pub version: String,
    pub branch: String,
    pub commits: Vec<Commit>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseStatus {
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Commit {
    pub sha: String,
    pub subject: String,
}

impl ReleaseRecord {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes =
            std::fs::read(path).with_context(|| format!("读取发布记录 {}", path.display()))?;
        let record: Self = serde_json::from_slice(&bytes)
            .with_context(|| format!("解析发布记录 {}", path.display()))?;
        record
            .validate()
            .with_context(|| format!("发布记录 {}", path.display()))?;
        Ok(record)
    }

    fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            bail!("不支持发布记录 schema_version {}", self.schema_version);
        }
        if self.release_id.trim().is_empty() {
            bail!("release_id 不能为空");
        }
        Ok(())
    }
}

/// Rendering has no Git, publishing, clipboard or filesystem write dependencies.
pub fn render(record: &ReleaseRecord, config: &ResolvedMessageConfig) -> Result<String> {
    record.validate()?;
    let (name, source) = match &config.template {
        Some(path) => (
            path.display().to_string(),
            std::fs::read_to_string(path)
                .with_context(|| format!("读取消息模板 {}", path.display()))?,
        ),
        None => ("内置消息模板".into(), DEFAULT_TEMPLATE.into()),
    };
    let releases: Vec<_> = record
        .releases
        .iter()
        .filter(|release| release.status == ReleaseStatus::Success)
        .collect();
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_keep_trailing_newline(true);
    env.add_template(&name, &source)
        .with_context(|| format!("解析消息模板 {name}"))?;
    env.get_template(&name)?
        .render(context! {
            releases => releases,
            variables => &config.variables,
            release_id => &record.release_id,
        })
        .with_context(|| format!("渲染消息模板 {name}"))
}
