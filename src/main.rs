use anyhow::{Context, Result, bail};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use rls::i18n::text as l;
use rls::{config, git, message};
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[command(version, disable_help_subcommand = true, about = l("多分支 tag 与发布消息", "Multi-branch tags and release messages"))]
struct Cli {
    #[arg(long, global = true, value_parser = ["en", "zh-CN", "zh"], help = l("界面语言", "Interface language"))]
    lang: Option<String>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 从已有 tag 生成消息，不创建或推送 tag
    #[command(visible_aliases = ["msg", "m"], about = l("从每个分支最近两个 tag 生成消息", "Generate messages between the latest two tags on each branch"))]
    Message {
        /// 分支列表，省略使用当前分支；每项取上一 tag 到最新 tag
        #[arg(help = l("分支列表，默认当前分支", "Branches; defaults to the current branch"))]
        branches: Vec<String>,
        /// 写入文件，省略时输出到终端
        #[arg(short, long, help = l("写入文件", "Write to a file"))]
        output: Option<PathBuf>,
        /// 同时复制到系统剪贴板
        #[arg(short, long, help = l("复制到剪贴板", "Copy to clipboard"))]
        copy: bool,
    },
    /// 按远端分支创建递增 tag
    #[command(visible_alias = "t", about = l("基于远端分支创建递增 tag", "Create incremental tags from remote branches"))]
    Tag {
        #[arg(help = l("分支列表，默认当前分支", "Branches; defaults to the current branch"))]
        branches: Vec<String>,
        #[arg(short = 'n', long)]
        #[arg(help = l("预览；会 fetch，不创建或推送 tag", "Preview; fetches refs without creating or pushing tags"))]
        dry_run: bool,
        #[arg(short = 'p', long)]
        #[arg(help = l("推送所选 tag", "Push selected tags"))]
        push: bool,
        #[arg(short = 'r', long)]
        #[arg(help = l("目标远端，默认 origin", "Remote; defaults to origin"))]
        remote: Option<String>,
        /// 显式名称，仅支持单分支；也用于失败后重试同一 tag
        #[arg(long)]
        #[arg(help = l("显式名称，仅单分支；用于首次发布或重试", "Explicit name for one branch; first release or retry"))]
        name: Option<String>,
    },
    /// 输出补全用的已配置分支（只读）
    #[command(hide = true)]
    CompletionBranches,
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    let mut language = None;
    for (i, arg) in args.iter().enumerate().skip(1) {
        if arg == "--" {
            break;
        }
        if arg == "--lang" {
            language = args.get(i + 1).and_then(|s| s.to_str());
        } else if let Some(value) = arg.to_str().and_then(|s| s.strip_prefix("--lang=")) {
            language = Some(value);
        }
    }
    rls::i18n::init(language)?;
    let mut command = Cli::command();
    if rls::i18n::is_zh() {
        command = command.help_template("{about}\n\n用法：{usage}\n\n{all-args}");
    }
    let matches = command.get_matches_from(args);
    match Cli::from_arg_matches(&matches)?.command {
        Commands::Message {
            branches,
            output,
            copy,
        } => {
            let cwd = std::env::current_dir()?;
            config::git_path(&cwd, "--git-common-dir")?
                .context(rls::tr!("请在目标 Git 仓库中生成消息"))?;
            let settings = config::load(&cwd, config::global_dir().as_deref())?;
            let branches = git::branches(&cwd, branches)?;
            let releases = branches
                .iter()
                .map(|branch| git::branch_release(&cwd, branch, &settings))
                .collect::<Result<Vec<_>>>()?;
            let rendered = message::render(&releases, &settings)?;
            if let Some(destination) = output {
                std::fs::write(&destination, rendered.as_bytes())
                    .with_context(|| rls::tr!("写入消息 {}", destination.display()))?;
            } else {
                std::io::stdout().lock().write_all(rendered.as_bytes())?;
            }
            if copy {
                arboard::Clipboard::new()
                    .context(rls::tr!("无法打开系统剪贴板"))?
                    .set_text(rendered)
                    .context(rls::tr!("复制消息失败，可重新执行 message --copy"))?;
            }
        }
        Commands::Tag {
            branches,
            dry_run,
            push,
            remote,
            name,
        } => {
            let cwd = std::env::current_dir()?;
            let settings = config::load(&cwd, config::global_dir().as_deref())?;
            let branches = git::branches(&cwd, branches)?;
            if name.is_some() && branches.len() != 1 {
                bail!("{}", rls::tr!("--name 仅支持单分支"));
            }
            let mut failed = false;
            for branch in branches {
                match git::create_tag(
                    &cwd,
                    &branch,
                    &settings,
                    remote.as_deref(),
                    name.as_deref(),
                    dry_run,
                    push,
                ) {
                    Ok(report) => println!("{branch}: {report}"),
                    Err(error) => {
                        eprintln!("{branch}: {error:#}");
                        failed = true;
                    }
                }
            }
            if failed {
                bail!(
                    "{}",
                    rls::tr!("部分分支失败；已创建的 tag 保留，使用 --name 重试原名称")
                );
            }
        }
        Commands::CompletionBranches => {
            let cwd = std::env::current_dir()?;
            let settings = config::load(&cwd, config::global_dir().as_deref())?;
            for branch in settings.branches.keys() {
                println!("{branch}");
            }
        }
    }
    Ok(())
}
