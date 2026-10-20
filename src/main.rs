use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rls::{
    config,
    message::{self, ReleaseRecord, ReleaseStatus},
};
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[command(version, about = "从发布记录生成可配置的发布消息")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 重新渲染已保存的记录，不执行发布动作
    Message {
        /// 当前仓库 .rls/state/releases/<release-id>.json 中的记录
        #[arg(required_unless_present = "record", conflicts_with = "record")]
        release_id: Option<String>,
        /// 显式读取 JSON 记录，也可在仓库外使用
        #[arg(long)]
        record: Option<PathBuf>,
        /// 写入文件，省略时输出到终端
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 同时复制到系统剪贴板
        #[arg(long)]
        copy: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Message {
            release_id,
            record,
            output,
            copy,
        } => {
            let cwd = std::env::current_dir()?;
            let path = match record {
                Some(path) => path,
                None => {
                    let id = release_id.as_deref().expect("clap requires release id");
                    if id.is_empty()
                        || !id
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
                    {
                        bail!("release-id 只能包含字母、数字、连字符和下划线");
                    }
                    let root = config::git_path(&cwd, "--show-toplevel")?
                        .context("仓库外请使用 --record 指定发布记录")?;
                    root.join(".rls/state/releases").join(format!("{id}.json"))
                }
            };
            let record = ReleaseRecord::load(&path)?;
            if let Some(id) = release_id
                && id != record.release_id
            {
                bail!("记录中的 release_id 与请求不符");
            }
            let settings = config::load(&cwd, config::global_dir().as_deref())?;
            let rendered = message::render(&record, &settings)?;
            for release in record
                .releases
                .iter()
                .filter(|r| r.status != ReleaseStatus::Success)
            {
                eprintln!("未纳入消息: {} ({:?})", release.tag, release.status);
            }
            if let Some(destination) = output {
                // Never overwrite the source record, including through symlinks/hard links.
                if destination.exists() && same_file::is_same_file(&path, &destination)? {
                    bail!("输出文件不能覆盖发布记录");
                }
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
