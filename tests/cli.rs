use std::process::Command;

fn cli(args: &[&str], language: Option<&str>, locale: &str) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rls"));
    command
        .args(args)
        .env_remove("RLS_LANG")
        .env_remove("LC_MESSAGES")
        .env("LC_ALL", locale);
    if let Some(language) = language {
        command.env("RLS_LANG", language);
    }
    command.output().unwrap()
}

#[test]
fn languages_and_precedence() {
    for args in [
        vec!["--lang", "en", "tag", "--help"],
        vec!["t", "--lang=en", "--help"],
    ] {
        let out = cli(&args, Some("zh-CN"), "zh_CN.UTF-8");
        assert!(out.status.success());
        assert!(String::from_utf8_lossy(&out.stdout).contains("Create incremental tags"));
    }
    let out = cli(&["msg", "--help"], None, "zh_CN.UTF-8");
    assert!(String::from_utf8_lossy(&out.stdout).contains("最近两个 tag"));
    let out = cli(&["msg", "--help"], Some("en"), "zh_CN.UTF-8");
    assert!(String::from_utf8_lossy(&out.stdout).contains("latest two tags"));
    let out = cli(&["--help"], Some("fr"), "C");
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Unsupported language"));
}

#[test]
fn localized_errors_do_not_rewrite_input() {
    let dir = tempfile::tempdir().unwrap();
    for (language, expected) in [
        ("en", "target Git repository"),
        ("zh-CN", "请在目标 Git 仓库"),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(dir.path())
            .env("RLS_LANG", language)
            .args(["msg", "main"])
            .output()
            .unwrap();
        assert!(!out.status.success());
        assert!(String::from_utf8_lossy(&out.stderr).contains(expected));
    }
}
