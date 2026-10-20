use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rls::{config, git, message};
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "从 Git tag 和提交生成可配置的发布消息")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 从已有 tag 生成消息，不创建或推送 tag
    #[command(visible_aliases = ["msg", "m"])]
    Message {
        /// 一个或多个目标 tag，按传入顺序输出
        #[arg(required = true, num_args = 1..)]
        tags: Vec<String>,
        /// 显式基线 tag，仅适用于一个目标 tag；省略时按仓库版本线自动寻找
        #[arg(long)]
        from: Option<String>,
        /// 写入文件，省略时输出到终端
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 同时复制到系统剪贴板
        #[arg(short, long)]
        copy: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Message {
            tags,
            from,
            output,
            copy,
        } => {
            if from.is_some() && tags.len() != 1 {
                bail!("--from 仅适用于一个目标 tag");
            }
            let cwd = std::env::current_dir()?;
            config::git_path(&cwd, "--git-common-dir")?.context("请在目标 Git 仓库中生成消息")?;
            let settings = config::load(&cwd, config::global_dir().as_deref())?;
            let releases = tags
                .iter()
                .map(|tag| git::release(&cwd, tag, from.as_deref(), &settings.branches))
                .collect::<Result<Vec<_>>>()?;
            let rendered = message::render(&releases, &settings)?;
            if let Some(destination) = output {
                std::fs::write(&destination, rendered.as_bytes())
                    .with_context(|| format!("写入消息 {}", destination.display()))?;
            } else {
                std::io::stdout().lock().write_all(rendered.as_bytes())?;
            }
            if copy {
                arboard::Clipboard::new()
                    .context("无法打开系统剪贴板")?
                    .set_text(rendered)
                    .context("复制消息失败，可重新执行 message --copy")?;
            }
        }
    }
    Ok(())
}
