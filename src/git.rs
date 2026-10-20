use crate::{
    config::BranchRule,
    message::{Commit, Release},
};
use anyhow::{Context, Result, bail};
use std::{
    path::Path,
    process::{Command, Output},
};

fn command(cwd: &Path, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .current_dir(cwd)
        .env("LC_ALL", "C")
        .args(["--no-pager", "--no-replace-objects"])
        .args(args)
        .output()
        .context(crate::tr!("无法执行 Git"))
}

fn text(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = command(cwd, args)?;
    if !output.status.success() {
        bail!(
            "{}",
            crate::tr!(
                "Git {} 失败: {}",
                args[0],
                String::from_utf8_lossy(&output.stderr).trim()
            )
        );
    }
    String::from_utf8(output.stdout).context(crate::tr!("Git 输出不是 UTF-8"))
}

fn tag_sha(cwd: &Path, tag: &str) -> Result<String> {
    let reference = format!("refs/tags/{tag}");
    if !command(cwd, &["check-ref-format", &reference])?
        .status
        .success()
    {
        bail!("{}", crate::tr!("非法 tag 名称: {tag}", tag = tag));
    }
    text(
        cwd,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{reference}^{{commit}}"),
        ],
    )
    .map(|sha| sha.trim().to_owned())
    .with_context(|| {
        crate::tr!(
            "找不到指向提交的 tag {tag}；请确认本地已有所需 tags",
            tag = tag
        )
    })
}

fn ancestor(cwd: &Path, old: &str, new: &str) -> Result<bool> {
    let output = command(cwd, &["merge-base", "--is-ancestor", old, new])?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => bail!(
            "{}",
            crate::tr!(
                "Git 祖先检查失败: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )
        ),
    }
}

fn number(tag: &str, rule: &BranchRule) -> Option<u64> {
    if rule.exclude_tags.iter().any(|excluded| excluded == tag) {
        return None;
    }
    let prefix = format!("{}{}.", rule.prefix(), rule.version_prefix.as_deref()?);
    let suffix = tag.strip_prefix(&prefix)?;
    if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    suffix.parse().ok()
}

fn release(cwd: &Path, tag: &str, from: &str, branch: &str, rule: &BranchRule) -> Result<Release> {
    let sha = tag_sha(cwd, tag)?;
    let version = tag.strip_prefix(rule.prefix()).unwrap_or(tag).to_owned();
    let (previous_tag, previous_sha) = (from.to_owned(), tag_sha(cwd, from)?);
    if !ancestor(cwd, &previous_sha, &sha)? {
        bail!(
            "{}",
            crate::tr!(
                "基线 {previous_tag} 不是目标 {tag} 的祖先",
                previous_tag = previous_tag,
                tag = tag
            )
        );
    }
    if previous_sha == sha {
        bail!(
            "{}",
            crate::tr!(
                "无新变更: {previous_tag} 与 {tag} 指向同一提交",
                previous_tag = previous_tag,
                tag = tag
            )
        );
    }
    let range = format!("{previous_sha}..{sha}");
    let log = text(
        cwd,
        &[
            "log",
            "--reverse",
            "--topo-order",
            "--no-show-signature",
            "--format=%H%x00%s",
            &range,
            "--",
        ],
    )?;
    let commits = log
        .lines()
        .map(|line| {
            let (sha, subject) = line
                .split_once('\0')
                .context(crate::tr!("无法解析 Git 提交输出"))?;
            Ok(Commit {
                sha: sha.into(),
                subject: subject.into(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Release {
        tag: tag.into(),
        previous_tag,
        sha,
        previous_sha,
        version,
        branch: branch.into(),
        commits,
    })
}

pub fn branches(cwd: &Path, mut names: Vec<String>) -> Result<Vec<String>> {
    if names.is_empty() {
        names.push(
            text(cwd, &["symbolic-ref", "--quiet", "--short", "HEAD"])
                .context(crate::tr!("detached HEAD 请显式指定分支"))?
                .trim()
                .into(),
        );
    }
    let mut seen = std::collections::BTreeSet::new();
    for name in &names {
        text(cwd, &["check-ref-format", &format!("refs/heads/{name}")])?;
        if !seen.insert(name) {
            bail!("{}", crate::tr!("重复分支: {name}", name = name));
        }
    }
    Ok(names)
}

fn history(cwd: &Path, target: &str, rule: &BranchRule) -> Result<Vec<(u64, String, String)>> {
    if text(cwd, &["rev-parse", "--is-shallow-repository"])?.trim() == "true" {
        bail!("{}", crate::tr!("浅克隆无法可靠选择上一 tag，请补全历史"));
    }
    let mut entries = Vec::new();
    for tag in text(
        cwd,
        &["for-each-ref", "--format=%(refname:strip=2)", "refs/tags/"],
    )?
    .lines()
    {
        if matches_rule(tag, rule) {
            let n = number(tag, rule).unwrap_or(0);
            let object = text(
                cwd,
                &[
                    "rev-parse",
                    "--verify",
                    "--end-of-options",
                    &format!("refs/tags/{tag}^{{}}"),
                ],
            )?;
            let sha = object.trim().to_owned();
            if text(cwd, &["cat-file", "-t", &sha])?.trim() != "commit" {
                continue;
            }
            if ancestor(cwd, &sha, target)? {
                entries.push((n, tag.to_owned(), sha));
            }
        }
    }
    if rule.tag_pattern.is_some() {
        // Rank each node by its number of strict ancestors. Every pair must be comparable.
        let mut ranks = vec![0; entries.len()];
        for i in 0..entries.len() {
            for j in i + 1..entries.len() {
                if entries[i].2 == entries[j].2 {
                    bail!(
                        "{}",
                        crate::tr!("版本基线有歧义: {} / {}", entries[i].1, entries[j].1)
                    );
                }
                if ancestor(cwd, &entries[i].2, &entries[j].2)? {
                    ranks[j] += 1;
                } else if ancestor(cwd, &entries[j].2, &entries[i].2)? {
                    ranks[i] += 1;
                } else {
                    bail!(
                        "{}",
                        crate::tr!("版本历史不单调或分叉: {} / {}", entries[i].1, entries[j].1)
                    );
                }
            }
        }
        for (entry, rank) in entries.iter_mut().zip(ranks) {
            entry.0 = rank;
        }
    }
    entries.sort_by_key(|item| item.0);
    for pair in entries.windows(2) {
        if pair[0].0 == pair[1].0 {
            bail!(
                "{}",
                crate::tr!("版本基线有歧义: {} / {}", pair[0].1, pair[1].1)
            );
        }
        if !ancestor(cwd, &pair[0].2, &pair[1].2)? {
            bail!(
                "{}",
                crate::tr!("版本历史不单调或分叉: {} / {}", pair[0].1, pair[1].1)
            );
        }
    }
    Ok(entries)
}

pub fn branch_release(
    cwd: &Path,
    branch: &str,
    settings: &crate::config::ResolvedMessageConfig,
) -> Result<Release> {
    let rule = settings
        .branches
        .get(branch)
        .with_context(|| crate::tr!("请配置 branches.{branch}", branch = branch))?;
    let local = format!("refs/heads/{branch}");
    let remote = format!("refs/remotes/{}/{branch}", settings.release.remote);
    let reference = if command(cwd, &["show-ref", "--verify", "--quiet", &local])?
        .status
        .success()
    {
        local
    } else {
        remote
    };
    let target = text(
        cwd,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{reference}^{{commit}}"),
        ],
    )?;
    let entries = history(cwd, target.trim(), rule)?;
    if entries.len() < 2 {
        bail!(
            "{}",
            crate::tr!("分支 {branch} 需要至少两个 tag", branch = branch)
        );
    }
    let latest = &entries[entries.len() - 1];
    let previous = &entries[entries.len() - 2];
    release(cwd, &latest.1, &previous.1, branch, rule)
}

#[allow(clippy::too_many_arguments)]
pub fn create_tag(
    cwd: &Path,
    branch: &str,
    settings: &crate::config::ResolvedMessageConfig,
    remote: Option<&str>,
    name: Option<&str>,
    dry_run: bool,
    push: bool,
) -> Result<String> {
    let rule = settings
        .branches
        .get(branch)
        .with_context(|| crate::tr!("请配置 branches.{branch}", branch = branch))?;
    if name.is_none() && rule.increment == crate::config::Increment::Script {
        crate::script::ensure_supported()?;
    }
    let remote = remote.unwrap_or(&settings.release.remote);
    // Only configured remote names: never interpret a URL or option from configuration.
    if !text(cwd, &["remote"])?.lines().any(|r| r == remote) || remote.starts_with('-') {
        bail!("{}", crate::tr!("不存在 remote {remote}", remote = remote));
    }
    let branch_ref = format!("refs/heads/{branch}");
    text(cwd, &["fetch", "--no-tags", remote, &branch_ref])?;
    let target = text(cwd, &["rev-parse", "--verify", "FETCH_HEAD^{commit}"])?
        .trim()
        .to_owned();
    text(
        cwd,
        &["fetch", "--no-tags", remote, "refs/tags/*:refs/tags/*"],
    )?;
    let entries = history(cwd, &target, rule)?;
    let tag = if let Some(name) = name {
        name.to_owned()
    } else {
        if rule.increment == crate::config::Increment::Manual {
            bail!("{}", crate::tr!("manual 模式需要 --name"));
        }
        if let Some(latest) = entries.last().filter(|latest| latest.2 == target) {
            return Ok(crate::tr!(
                "无新变更 ({})；如需补推请使用 --name {} --push",
                latest.1,
                latest.1
            ));
        }
        if rule.increment == crate::config::Increment::Script {
            let root = text(cwd, &["rev-parse", "--show-toplevel"])?;
            let request = crate::script::Request {
                version: 1,
                branch,
                target_sha: &target,
                previous_tag: entries.last().map(|e| e.1.as_str()),
                previous_sha: entries.last().map(|e| e.2.as_str()),
                dry_run,
            };
            match crate::script::run(
                Path::new(root.trim_end_matches(['\r', '\n'])),
                rule.script.as_ref().expect("validated script"),
                &request,
            )? {
                crate::script::Decision::Tag(name) => name,
                crate::script::Decision::Skip(reason) => {
                    return Ok(format!(
                        "{}: {reason}",
                        crate::i18n::text("跳过", "Skipped")
                    ));
                }
            }
        } else {
            let latest = entries
                .last()
                .context(crate::tr!("没有基线 tag，首次发布请使用 --name"))?;
            let n = latest
                .0
                .checked_add(1)
                .context(crate::tr!("tag 计数溢出"))?;
            format!(
                "{}{}.{}",
                rule.prefix(),
                rule.version_prefix
                    .as_deref()
                    .expect("validated numeric rule"),
                n
            )
        }
    };
    if !matches_rule(&tag, rule) {
        bail!(
            "{}",
            crate::i18n::text(
                "tag 必须匹配分支规则且不能被排除",
                "tag must match the branch rule and must not be excluded"
            )
        );
    }
    let reference = format!("refs/tags/{tag}");
    text(cwd, &["check-ref-format", &reference])?;
    let exists = command(cwd, &["show-ref", "--verify", "--quiet", &reference])?
        .status
        .success();
    if exists {
        if tag_sha(cwd, &tag)? != target {
            bail!(
                "{}",
                crate::tr!("tag {tag} 已指向其他提交，拒绝覆盖", tag = tag)
            );
        }
    } else if let Some(latest) = entries.last() {
        if rule.tag_pattern.is_none() && number(&tag, rule).unwrap() <= latest.0 {
            bail!("{}", crate::tr!("新 tag 必须大于上一版本 {}", latest.1));
        }
        if latest.2 == target {
            bail!("{}", crate::tr!("无新变更，拒绝创建空版本"));
        }
    }
    if dry_run {
        return Ok(crate::tr!(
            "预览 {tag} -> {target}{}",
            if push { " (push)" } else { "" },
            tag = tag,
            target = target
        ));
    }
    let current = text(cwd, &["ls-remote", "--heads", remote, &branch_ref])?;
    if current.split_whitespace().next() != Some(target.as_str()) {
        bail!("{}", crate::tr!("远端分支已变化，请重新执行"));
    }
    if !exists {
        // Compare-and-set creates a lightweight tag without replacing concurrent changes.
        text(cwd, &["update-ref", &reference, &target, ""])?;
    }
    if push {
        text(cwd, &["push", remote, &format!("{reference}:{reference}")]).with_context(|| {
            format!(
                "{}: rls tag {} --name {} --remote {} --push",
                crate::i18n::text(
                    "本地 tag 已保留，按原名称重试",
                    "Local tag retained; retry the original name"
                ),
                shell_argument(branch),
                shell_argument(&tag),
                shell_argument(remote)
            )
        })?;
    }
    Ok(format!(
        "{tag} -> {target} ({})",
        if push {
            crate::tr!("已推送")
        } else if exists {
            crate::tr!("已存在")
        } else {
            crate::tr!("已创建")
        }
    ))
}

fn matches_rule(tag: &str, rule: &BranchRule) -> bool {
    !rule.exclude_tags.iter().any(|excluded| excluded == tag)
        && match &rule.tag_pattern {
            Some(pattern) => glob_matches(pattern, tag),
            None => number(tag, rule).is_some(),
        }
}

// Whole-name glob with only '*' and '?'; '?' matches one Unicode scalar, including '/'.
fn glob_matches(pattern: &str, name: &str) -> bool {
    let name: Vec<_> = name.chars().collect();
    let mut previous = vec![false; name.len() + 1];
    previous[0] = true;
    for token in pattern.chars() {
        let mut next = vec![false; name.len() + 1];
        if token == '*' {
            next[0] = previous[0];
        }
        for i in 1..=name.len() {
            next[i] = if token == '*' {
                previous[i] || next[i - 1]
            } else {
                previous[i - 1] && (token == '?' || token == name[i - 1])
            };
        }
        previous = next;
    }
    previous[name.len()]
}

fn shell_argument(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._/-".contains(&b))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\"'\"'"))
    }
}
