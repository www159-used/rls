use crate::{
    config::BranchRule,
    message::{Commit, Release},
};
use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeMap,
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
        .context("无法执行 Git")
}

fn text(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = command(cwd, args)?;
    if !output.status.success() {
        bail!(
            "Git {} 失败: {}",
            args[0],
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    String::from_utf8(output.stdout).context("Git 输出不是 UTF-8")
}

fn tag_sha(cwd: &Path, tag: &str) -> Result<String> {
    let reference = format!("refs/tags/{tag}");
    if !command(cwd, &["check-ref-format", &reference])?
        .status
        .success()
    {
        bail!("非法 tag 名称: {tag}");
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
    .with_context(|| format!("找不到指向提交的 tag {tag}；请确认本地已有所需 tags"))
}

fn ancestor(cwd: &Path, old: &str, new: &str) -> Result<bool> {
    let output = command(cwd, &["merge-base", "--is-ancestor", old, new])?;
    match output.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => bail!(
            "Git 祖先检查失败: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ),
    }
}

fn number(tag: &str, rule: &BranchRule) -> Option<u64> {
    let prefix = format!("{}{}.", rule.tag_prefix, rule.version_prefix);
    let suffix = tag.strip_prefix(&prefix)?;
    if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    suffix.parse().ok()
}

fn previous(
    cwd: &Path,
    tag: &str,
    target_sha: &str,
    rule: &BranchRule,
    target_number: u64,
) -> Result<(String, String)> {
    let tags = text(
        cwd,
        &["for-each-ref", "--format=%(refname:strip=2)", "refs/tags/"],
    )?;
    let mut candidates = Vec::new();
    for candidate in tags.lines().filter(|candidate| *candidate != tag) {
        let Some(n) = number(candidate, rule) else {
            continue;
        };
        let sha = tag_sha(cwd, candidate)?;
        if !ancestor(cwd, &sha, target_sha)? {
            continue;
        }
        if n >= target_number {
            bail!("版本历史不单调: {candidate} 是 {tag} 的祖先；请用 --from 显式指定基线");
        }
        candidates.push((n, candidate.to_owned(), sha));
    }
    candidates.sort_by_key(|item| item.0);
    for pair in candidates.windows(2) {
        if pair[0].0 == pair[1].0 || !ancestor(cwd, &pair[0].2, &pair[1].2)? {
            bail!(
                "版本基线有歧义: {} 与 {}；请用 --from 显式指定基线",
                pair[0].1,
                pair[1].1
            );
        }
    }
    let (_, name, sha) = candidates
        .pop()
        .context("没有上一 tag；请用 --from 显式指定基线")?;
    Ok((name, sha))
}

pub fn release(
    cwd: &Path,
    tag: &str,
    from: Option<&str>,
    branches: &BTreeMap<String, BranchRule>,
) -> Result<Release> {
    let sha = tag_sha(cwd, tag)?;
    let matching: Vec<_> = branches
        .iter()
        .filter_map(|(branch, rule)| number(tag, rule).map(|n| (branch, rule, n)))
        .collect();
    if matching.len() > 1 {
        bail!("tag {tag} 匹配多个分支版本线，请修正仓库配置");
    }
    let (branch, version) = matching
        .first()
        .map(|(branch, rule, _)| {
            (
                (*branch).clone(),
                tag.strip_prefix(&rule.tag_prefix).unwrap().to_owned(),
            )
        })
        .unwrap_or_else(|| (String::new(), tag.to_owned()));
    let (previous_tag, previous_sha) = if let Some(from) = from {
        (from.to_owned(), tag_sha(cwd, from)?)
    } else {
        let (_, rule, n) = matching
            .first()
            .context("目标 tag 未匹配仓库版本线；请配置 branches 或使用 --from")?;
        if text(cwd, &["rev-parse", "--is-shallow-repository"])?.trim() == "true" {
            bail!("浅克隆无法可靠选择上一 tag，请补全历史或用 --from 指定可验证的基线");
        }
        previous(cwd, tag, &sha, rule, *n)?
    };
    if !ancestor(cwd, &previous_sha, &sha)? {
        bail!("基线 {previous_tag} 不是目标 {tag} 的祖先");
    }
    if previous_sha == sha {
        bail!("无新变更: {previous_tag} 与 {tag} 指向同一提交");
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
            let (sha, subject) = line.split_once('\0').context("无法解析 Git 提交输出")?;
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
        branch,
        commits,
    })
}
