use std::sync::OnceLock;
static CHINESE: OnceLock<bool> = OnceLock::new();

/// Precedence: explicit CLI choice, RLS_LANG, LC_ALL, LC_MESSAGES, LANG, English.
pub fn init(explicit: Option<&str>) -> anyhow::Result<()> {
    let configured = explicit
        .map(str::to_owned)
        .or_else(|| std::env::var("RLS_LANG").ok().filter(|s| !s.is_empty()));
    let zh = if let Some(language) = configured {
        match language.as_str() {
            "zh" | "zh-CN" => true,
            "en" => false,
            _ => anyhow::bail!("Unsupported language / 不支持的语言: {language}; use en or zh-CN"),
        }
    } else {
        ["LC_ALL", "LC_MESSAGES", "LANG"]
            .iter()
            .find_map(|key| std::env::var(key).ok().filter(|s| !s.is_empty()))
            .is_some_and(|locale| locale.to_ascii_lowercase().starts_with("zh"))
    };
    let _ = CHINESE.set(zh);
    Ok(())
}
pub fn is_zh() -> bool {
    *CHINESE.get_or_init(|| false)
}
pub fn text(zh: &'static str, en: &'static str) -> &'static str {
    if is_zh() { zh } else { en }
}

// Message catalog. Both translations are checked by Rust's format! at compile time.
#[macro_export]
macro_rules! tr {
    ("无法运行 git rev-parse" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无法运行 git rev-parse" $(, $($args)*)?) } else { format!("Cannot run git rev-parse" $(, $($args)*)?) } };
    ("Git 目录发现失败: {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("Git 目录发现失败: {}" $(, $($args)*)?) } else { format!("Git directory discovery failed: {}" $(, $($args)*)?) } };
    ("Git 目录不是 UTF-8 路径" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("Git 目录不是 UTF-8 路径" $(, $($args)*)?) } else { format!("Git directory is not a UTF-8 path" $(, $($args)*)?) } };
    ("读取配置 {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("读取配置 {}" $(, $($args)*)?) } else { format!("Read configuration {}" $(, $($args)*)?) } };
    ("解析配置 {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("解析配置 {}" $(, $($args)*)?) } else { format!("Parse configuration {}" $(, $($args)*)?) } };
    ("配置 {} 不支持 schema_version {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("配置 {} 不支持 schema_version {}" $(, $($args)*)?) } else { format!("Configuration {} has unsupported schema_version {}" $(, $($args)*)?) } };
    ("配置 {} 不支持 message.version {}，当前仅支持 1" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("配置 {} 不支持 message.version {}，当前仅支持 1" $(, $($args)*)?) } else { format!("Configuration {} has unsupported message.version {}; only 1 is supported" $(, $($args)*)?) } };
    ("配置 {} 的 message.template 不能为空" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("配置 {} 的 message.template 不能为空" $(, $($args)*)?) } else { format!("Configuration {} has an empty message.template" $(, $($args)*)?) } };
    ("配置 {} 的分支 {name} version_prefix 必须为点分数字" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("配置 {} 的分支 {name} version_prefix 必须为点分数字" $(, $($args)*)?) } else { format!("Configuration {} branch {name}: version_prefix must contain dot-separated numbers" $(, $($args)*)?) } };
    ("无法执行 Git" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无法执行 Git" $(, $($args)*)?) } else { format!("Cannot execute Git" $(, $($args)*)?) } };
    ("Git {} 失败: {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("Git {} 失败: {}" $(, $($args)*)?) } else { format!("Git {} failed: {}" $(, $($args)*)?) } };
    ("Git 输出不是 UTF-8" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("Git 输出不是 UTF-8" $(, $($args)*)?) } else { format!("Git output is not UTF-8" $(, $($args)*)?) } };
    ("非法 tag 名称: {tag}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("非法 tag 名称: {tag}" $(, $($args)*)?) } else { format!("Invalid tag name: {tag}" $(, $($args)*)?) } };
    ("找不到指向提交的 tag {tag}；请确认本地已有所需 tags" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("找不到指向提交的 tag {tag}；请确认本地已有所需 tags" $(, $($args)*)?) } else { format!("No commit tag {tag}; ensure the required tags exist locally" $(, $($args)*)?) } };
    ("Git 祖先检查失败: {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("Git 祖先检查失败: {}" $(, $($args)*)?) } else { format!("Git ancestry check failed: {}" $(, $($args)*)?) } };
    ("tag {tag} 匹配多个分支版本线，请修正仓库配置" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("tag {tag} 匹配多个分支版本线，请修正仓库配置" $(, $($args)*)?) } else { format!("Tag {tag} matches multiple branch rules; fix repository configuration" $(, $($args)*)?) } };
    ("基线 {previous_tag} 不是目标 {tag} 的祖先" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("基线 {previous_tag} 不是目标 {tag} 的祖先" $(, $($args)*)?) } else { format!("Baseline {previous_tag} is not an ancestor of {tag}" $(, $($args)*)?) } };
    ("无新变更: {previous_tag} 与 {tag} 指向同一提交" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无新变更: {previous_tag} 与 {tag} 指向同一提交" $(, $($args)*)?) } else { format!("No changes: {previous_tag} and {tag} point to the same commit" $(, $($args)*)?) } };
    ("无法解析 Git 提交输出" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无法解析 Git 提交输出" $(, $($args)*)?) } else { format!("Cannot parse Git commit output" $(, $($args)*)?) } };
    ("detached HEAD 请显式指定分支" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("detached HEAD 请显式指定分支" $(, $($args)*)?) } else { format!("Detached HEAD: specify branches explicitly" $(, $($args)*)?) } };
    ("重复分支: {name}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("重复分支: {name}" $(, $($args)*)?) } else { format!("Duplicate branch: {name}" $(, $($args)*)?) } };
    ("浅克隆无法可靠选择上一 tag，请补全历史" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("浅克隆无法可靠选择上一 tag，请补全历史" $(, $($args)*)?) } else { format!("Cannot select tags from a shallow clone; fetch complete history" $(, $($args)*)?) } };
    ("版本基线有歧义: {} / {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("版本基线有歧义: {} / {}" $(, $($args)*)?) } else { format!("Ambiguous tag history: {} / {}" $(, $($args)*)?) } };
    ("版本历史不单调或分叉: {} / {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("版本历史不单调或分叉: {} / {}" $(, $($args)*)?) } else { format!("Non-monotonic or divergent tag history: {} / {}" $(, $($args)*)?) } };
    ("请配置 branches.{branch}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("请配置 branches.{branch}" $(, $($args)*)?) } else { format!("Configure branches.{branch}" $(, $($args)*)?) } };
    ("分支 {branch} 需要至少两个 tag" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("分支 {branch} 需要至少两个 tag" $(, $($args)*)?) } else { format!("Branch {branch} requires at least two tags" $(, $($args)*)?) } };
    ("不存在 remote {remote}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("不存在 remote {remote}" $(, $($args)*)?) } else { format!("Remote {remote} does not exist" $(, $($args)*)?) } };
    ("--name 必须匹配分支版本线" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("--name 必须匹配分支版本线" $(, $($args)*)?) } else { format!("--name must match the branch version rule" $(, $($args)*)?) } };
    ("manual 模式需要 --name" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("manual 模式需要 --name" $(, $($args)*)?) } else { format!("Manual mode requires --name" $(, $($args)*)?) } };
    ("没有基线 tag，首次发布请使用 --name" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("没有基线 tag，首次发布请使用 --name" $(, $($args)*)?) } else { format!("No baseline tag; use --name for the first release" $(, $($args)*)?) } };
    ("无新变更 ({})；如需补推请使用 --name {} --push" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无新变更 ({})；如需补推请使用 --name {} --push" $(, $($args)*)?) } else { format!("No changes ({}); to retry pushing, use --name {} --push" $(, $($args)*)?) } };
    ("tag 计数溢出" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("tag 计数溢出" $(, $($args)*)?) } else { format!("Tag counter overflow" $(, $($args)*)?) } };
    ("tag {tag} 已指向其他提交，拒绝覆盖" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("tag {tag} 已指向其他提交，拒绝覆盖" $(, $($args)*)?) } else { format!("Tag {tag} points to another commit; refusing to overwrite" $(, $($args)*)?) } };
    ("新 tag 必须大于上一版本 {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("新 tag 必须大于上一版本 {}" $(, $($args)*)?) } else { format!("New tag must be greater than the previous version {}" $(, $($args)*)?) } };
    ("无新变更，拒绝创建空版本" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无新变更，拒绝创建空版本" $(, $($args)*)?) } else { format!("No changes; refusing to create an empty release" $(, $($args)*)?) } };
    ("预览 {tag} -> {target}{}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("预览 {tag} -> {target}{}" $(, $($args)*)?) } else { format!("Preview {tag} -> {target}{}" $(, $($args)*)?) } };
    ("远端分支已变化，请重新执行" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("远端分支已变化，请重新执行" $(, $($args)*)?) } else { format!("Remote branch changed; run the command again" $(, $($args)*)?) } };
    ("本地 {tag} 已保留；使用 tag {branch} --name {tag} --push 重试" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("本地 {tag} 已保留；使用 tag {branch} --name {tag} --push 重试" $(, $($args)*)?) } else { format!("Local tag {tag} retained; retry with tag {branch} --name {tag} --push" $(, $($args)*)?) } };
    ("已推送" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("已推送" $(, $($args)*)?) } else { format!("pushed" $(, $($args)*)?) } };
    ("已存在" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("已存在" $(, $($args)*)?) } else { format!("already exists" $(, $($args)*)?) } };
    ("已创建" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("已创建" $(, $($args)*)?) } else { format!("created" $(, $($args)*)?) } };
    ("读取消息模板 {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("读取消息模板 {}" $(, $($args)*)?) } else { format!("Read message template {}" $(, $($args)*)?) } };
    ("内置消息模板" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("内置消息模板" $(, $($args)*)?) } else { format!("built-in message template" $(, $($args)*)?) } };
    ("解析消息模板 {name}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("解析消息模板 {name}" $(, $($args)*)?) } else { format!("Parse message template {name}" $(, $($args)*)?) } };
    ("渲染消息模板 {name}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("渲染消息模板 {name}" $(, $($args)*)?) } else { format!("Render message template {name}" $(, $($args)*)?) } };
    ("请在目标 Git 仓库中生成消息" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("请在目标 Git 仓库中生成消息" $(, $($args)*)?) } else { format!("Run this command inside the target Git repository" $(, $($args)*)?) } };
    ("写入消息 {}" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("写入消息 {}" $(, $($args)*)?) } else { format!("Write message {}" $(, $($args)*)?) } };
    ("无法打开系统剪贴板" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("无法打开系统剪贴板" $(, $($args)*)?) } else { format!("Cannot open the system clipboard" $(, $($args)*)?) } };
    ("复制消息失败，可重新执行 message --copy" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("复制消息失败，可重新执行 message --copy" $(, $($args)*)?) } else { format!("Copy failed; retry message --copy" $(, $($args)*)?) } };
    ("--name 仅支持单分支" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("--name 仅支持单分支" $(, $($args)*)?) } else { format!("--name supports exactly one branch" $(, $($args)*)?) } };
    ("部分分支失败；已创建的 tag 保留，使用 --name 重试原名称" $(, $($args:tt)*)?) => { if $crate::i18n::is_zh() { format!("部分分支失败；已创建的 tag 保留，使用 --name 重试原名称" $(, $($args)*)?) } else { format!("Some branches failed; created tags are retained, use --name to retry the original tag" $(, $($args)*)?) } };
}
