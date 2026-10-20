//! Versioned naming protocol; scripts never own Git mutation.
use crate::{config::ScriptConfig, i18n::text as l};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize)]
pub struct Request<'a> {
    pub version: u32,
    pub branch: &'a str,
    pub target_sha: &'a str,
    pub previous_tag: Option<&'a str>,
    pub previous_sha: Option<&'a str>,
    pub dry_run: bool,
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Tag(String),
    Skip(String),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    version: u32,
    action: Action,
    // Explicit nulls are rejected as well as unexpected keys.
    #[serde(default, deserialize_with = "required_string")]
    name: Option<String>,
    #[serde(default, deserialize_with = "required_string")]
    reason: Option<String>,
}
fn required_string<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Action {
    Tag,
    Skip,
}

fn parse(bytes: &[u8]) -> Result<Decision> {
    let response: Response = serde_json::from_slice(bytes).context(l(
        "脚本响应不是有效的 v1 JSON",
        "Invalid script protocol JSON",
    ))?;
    if response.version != 1 {
        bail!(
            "{}: {}",
            l(
                "不支持的脚本协议版本",
                "Unsupported script protocol version"
            ),
            response.version
        );
    }
    match (response.action, response.name, response.reason) {
        (Action::Tag, Some(name), None) if !name.trim().is_empty() => Ok(Decision::Tag(name)),
        (Action::Skip, None, Some(reason)) if !reason.trim().is_empty() => {
            Ok(Decision::Skip(reason))
        }
        _ => bail!(
            "{}",
            l(
                "脚本 tag/name 或 skip/reason 字段无效",
                "Invalid script tag/name or skip/reason fields"
            )
        ),
    }
}

pub fn ensure_supported() -> Result<()> {
    if !cfg!(unix) {
        bail!(
            "{}",
            l(
                "此平台不支持 script 执行，请使用 Unix 或 --name",
                "Script execution requires Unix; use --name on this platform"
            )
        );
    }
    Ok(())
}

pub fn run(root: &Path, config: &ScriptConfig, request: &Request<'_>) -> Result<Decision> {
    ensure_supported()?;
    #[cfg(unix)]
    {
        unix::run(root, config, request)
    }
    #[cfg(not(unix))]
    {
        let _ = (root, config, request);
        unreachable!("platform checked")
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use std::{
        collections::VecDeque,
        io::{Read, Write},
        os::{fd::AsRawFd, unix::process::CommandExt},
        process::{Child, Command, Stdio},
        time::{Duration, Instant},
    };
    const LIMIT: usize = 64 * 1024;

    struct Process(Child);
    impl Drop for Process {
        fn drop(&mut self) {
            // process_group(0) makes the child's PID its PGID. Kill the group even
            // when the leader exited, since descendants may still hold pipes open.
            unsafe {
                libc::kill(-(self.0.id() as i32), libc::SIGKILL);
            }
            let _ = self.0.wait();
        }
    }
    fn nonblocking(pipe: &impl AsRawFd) -> Result<()> {
        let fd = pipe.as_raw_fd();
        // SAFETY: fd belongs to the live pipe; fcntl does not take ownership.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags == -1 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1
        {
            return Err(std::io::Error::last_os_error().into());
        }
        Ok(())
    }
    fn read_chunk(
        pipe: &mut impl Read,
        sink: &mut impl FnMut(&[u8]) -> Result<()>,
    ) -> Result<bool> {
        let mut buffer = [0; 8192];
        // One bounded read per iteration keeps stderr floods from starving the deadline.
        match pipe.read(&mut buffer) {
            Ok(0) => Ok(true),
            Ok(n) => {
                sink(&buffer[..n])?;
                Ok(false)
            }
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                Ok(false)
            }
            Err(e) => Err(e.into()),
        }
    }
    pub(super) fn run(
        root: &Path,
        config: &ScriptConfig,
        request: &Request<'_>,
    ) -> Result<Decision> {
        let start = Instant::now();
        let mut input = serde_json::to_vec(request)?;
        input.push(b'\n');
        let program = &config.command[0];
        let executable = if program.contains('/') {
            root.join(program)
        } else {
            program.into()
        };
        let child = Command::new(executable)
            .args(&config.command[1..])
            .current_dir(root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .context(l("无法启动脚本", "Cannot start script"))?;
        let mut process = Process(child);
        let mut stdin = Some(process.0.stdin.take().expect("piped stdin"));
        let mut stdout = process.0.stdout.take().expect("piped stdout");
        let mut stderr = process.0.stderr.take().expect("piped stderr");
        nonblocking(stdin.as_ref().unwrap())?;
        nonblocking(&stdout)?;
        nonblocking(&stderr)?;
        let (mut sent, mut output, mut logs) = (0, Vec::new(), VecDeque::new());
        let (mut out_closed, mut err_closed, mut truncated) = (false, false, false);
        let result = (|| -> Result<Decision> {
            loop {
                if start.elapsed() >= Duration::from_secs(config.timeout_seconds) {
                    bail!("{}", l("脚本执行超时", "Script timed out"));
                }
                if let Some(pipe) = stdin.as_mut() {
                    match pipe.write(&input[sent..]) {
                        Ok(0) => bail!("{}", l("脚本提前关闭 stdin", "Script closed stdin early")),
                        Ok(n) => {
                            sent += n;
                            if sent == input.len() {
                                stdin.take();
                            }
                        }
                        Err(e)
                            if matches!(
                                e.kind(),
                                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                            ) => {}
                        Err(e) => {
                            return Err(e)
                                .context(l("写入脚本 stdin 失败", "Writing script stdin failed"));
                        }
                    }
                }
                if !out_closed {
                    out_closed = read_chunk(&mut stdout, &mut |chunk| {
                        if output.len() + chunk.len() > LIMIT {
                            bail!(
                                "{}",
                                l("脚本 stdout 超过 64 KiB", "Script stdout exceeds 64 KiB")
                            );
                        }
                        output.extend_from_slice(chunk);
                        Ok(())
                    })?;
                }
                if !err_closed {
                    err_closed = read_chunk(&mut stderr, &mut |chunk| {
                        for byte in chunk {
                            if logs.len() == LIMIT {
                                logs.pop_front();
                                truncated = true;
                            }
                            logs.push_back(*byte);
                        }
                        Ok(())
                    })?;
                }
                if let Some(status) = process.0.try_wait()?
                    && out_closed
                    && err_closed
                {
                    if !status.success() {
                        bail!(
                            "{}: {status}",
                            l("脚本退出失败", "Script exited unsuccessfully")
                        );
                    }
                    if sent != input.len() {
                        bail!("{}", l("脚本提前关闭 stdin", "Script closed stdin early"));
                    }
                    return parse(&output);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
        })();
        // Kill/reap before reporting; no reader threads can outlive the deadline.
        drop(process);
        if !logs.is_empty() {
            let bytes: Vec<_> = logs.into_iter().collect();
            eprintln!(
                "{}: {}{}\n{}",
                request.branch,
                l("脚本 stderr", "Script stderr"),
                if truncated {
                    l("（已截断）", " (truncated)")
                } else {
                    ""
                },
                String::from_utf8_lossy(&bytes)
            );
        }
        result.with_context(|| {
            format!(
                "{}: {}",
                l("脚本命名失败", "Script naming failed"),
                request.branch
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_protocol() {
        assert_eq!(
            parse(br#"{"version":1,"action":"tag","name":"release-a"}"#).unwrap(),
            Decision::Tag("release-a".into())
        );
        for invalid in [
            r#"{"version":1,"version":1,"action":"tag","name":"x"}"#,
            r#"{"version":1,"action":"tag","name":"x","name":"y"}"#,
            r#"{"version":1,"action":"tag","name":"x","reason":null}"#,
            r#"{"version":1,"action":"skip","reason":" "}"#,
            r#"{"version":2,"action":"tag","name":"x"}"#,
            r#"{"version":1,"action":"tag","name":"x","push":true}"#,
            r#"{"version":1,"action":"tag","name":"x"} {}"#,
            r#"{"version":"1","action":"tag","name":"x"}"#,
        ] {
            assert!(parse(invalid.as_bytes()).is_err(), "{invalid}");
        }
    }
}

#[cfg(all(test, unix))]
mod process_tests {
    use super::*;
    use std::{
        os::unix::fs::PermissionsExt,
        time::{Duration, Instant},
    };

    fn execute(dir: &Path, body: &str) -> Result<Decision> {
        let path = dir.join("name.sh");
        std::fs::write(&path, format!("#!/bin/sh\ncat >/dev/null\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        let config = ScriptConfig {
            command: vec!["./name.sh".into()],
            timeout_seconds: 1,
        };
        run(
            dir,
            &config,
            &Request {
                version: 1,
                branch: "main",
                target_sha: "abc",
                previous_tag: None,
                previous_sha: None,
                dry_run: true,
            },
        )
    }

    #[test]
    fn kills_descendants_when_leader_exits_and_pipes_remain_open() {
        let dir = tempfile::tempdir().unwrap();
        let start = Instant::now();
        let result = execute(
            dir.path(),
            "(sleep 2; touch escaped) &\nprintf '%s' '{\"version\":1,\"action\":\"skip\",\"reason\":\"ok\"}'",
        );
        assert!(format!("{:#}", result.unwrap_err()).contains("timed out"));
        assert!(start.elapsed() < Duration::from_secs(2));
        std::thread::sleep(Duration::from_millis(1300));
        assert!(!dir.path().join("escaped").exists());
    }

    #[test]
    fn drains_large_stderr_and_accepts_exact_stdout_limit() {
        let dir = tempfile::tempdir().unwrap();
        let result = execute(
            dir.path(),
            "python3 -c 'import sys; sys.stderr.write(\"x\"*100000); s=\"{\\\"version\\\":1,\\\"action\\\":\\\"skip\\\",\\\"reason\\\":\\\"ok\\\"}\"; sys.stdout.write(s+\" \"*(65536-len(s)))'",
        );
        assert_eq!(result.unwrap(), Decision::Skip("ok".into()));
    }
}
