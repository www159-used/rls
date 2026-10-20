use crate::config::ResolvedMessageConfig;
use anyhow::{Context, Result};
use minijinja::{Environment, UndefinedBehavior, context};
use serde::Serialize;

const DEFAULT_TEMPLATE: &str = "{% for release in releases %}{{ release.tag }}\nchangelog:\n{% for commit in release.commits %}{{ commit.subject }}\n{% endfor %}{% if not loop.last %}\n\n{% endif %}{% endfor %}";

#[derive(Debug, Serialize)]
pub struct Release {
    pub tag: String,
    pub previous_tag: String,
    pub sha: String,
    pub previous_sha: String,
    pub version: String,
    pub branch: String,
    pub commits: Vec<Commit>,
}

#[derive(Debug, Serialize)]
pub struct Commit {
    pub sha: String,
    pub subject: String,
}

/// Rendering has no Git, publishing, clipboard or filesystem write dependencies.
pub fn render(releases: &[Release], config: &ResolvedMessageConfig) -> Result<String> {
    let (name, source) = match &config.template {
        Some(path) => (
            path.display().to_string(),
            std::fs::read_to_string(path)
                .with_context(|| crate::tr!("读取消息模板 {}", path.display()))?,
        ),
        None => (crate::tr!("内置消息模板"), DEFAULT_TEMPLATE.into()),
    };
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_keep_trailing_newline(true);
    env.add_template(&name, &source)
        .with_context(|| crate::tr!("解析消息模板 {name}", name = name))?;
    env.get_template(&name)?
        .render(context! {
            releases => releases,
            variables => &config.variables,
        })
        .with_context(|| crate::tr!("渲染消息模板 {name}", name = name))
}
