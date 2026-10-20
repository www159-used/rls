use crate::ReleaseWorld;
use cucumber::{gherkin::Step, given, then, when};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

#[derive(Debug)]
pub struct Fixture {
    _temp: tempfile::TempDir,
    repo: PathBuf,
    global: PathBuf,
    cwd: PathBuf,
    record: PathBuf,
    output: Option<Output>,
    original_record: Vec<u8>,
}

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(cwd)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fixture(world: &mut ReleaseWorld) -> &mut Fixture {
    world.fixture.as_mut().unwrap()
}

#[given("隔离的仓库、全局目录和发布记录")]
fn setup(world: &mut ReleaseWorld) {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let global = temp.path().join("global");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&global).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["commit", "--allow-empty", "-qm", "initial"]);
    let record = repo.join(".rls/state/releases/demo.json");
    std::fs::create_dir_all(record.parent().unwrap()).unwrap();
    let original_record = include_bytes!("../../examples/release-record.json").to_vec();
    std::fs::write(&record, &original_record).unwrap();
    world.fixture = Some(Fixture {
        _temp: temp,
        cwd: repo.clone(),
        repo,
        global,
        record,
        output: None,
        original_record,
    });
}

#[given(expr = "{word}配置为")]
fn config(world: &mut ReleaseWorld, layer: String, step: &Step) {
    let f = fixture(world);
    let path = match layer.as_str() {
        "全局" => f.global.join("config.toml"),
        "仓库" => f.repo.join(".git/rls/config.toml"),
        _ => panic!("unknown layer"),
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, step.docstring.as_ref().unwrap()).unwrap();
}

#[given(expr = "{word}模板为")]
fn template(world: &mut ReleaseWorld, layer: String, step: &Step) {
    let f = fixture(world);
    let root = match layer.as_str() {
        "全局" => f.global.clone(),
        "仓库" => f.repo.join(".git/rls"),
        _ => panic!("unknown layer"),
    };
    std::fs::create_dir_all(root.join("templates")).unwrap();
    std::fs::write(
        root.join("templates/message.j2"),
        step.docstring.as_ref().unwrap(),
    )
    .unwrap();
}

#[given("从仓库子目录执行")]
fn subdir(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f.repo.join("nested/child");
    std::fs::create_dir_all(&f.cwd).unwrap();
}

#[given("从 linked worktree 执行")]
fn worktree(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f._temp.path().join("worktree");
    git(
        &f.repo,
        &["worktree", "add", "--detach", f.cwd.to_str().unwrap()],
    );
}

#[given("从仓库外执行")]
fn outside(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f._temp.path().into();
}

#[when("生成消息")]
fn run(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .arg("message")
            .arg("--record")
            .arg(&f.record)
            .output()
            .unwrap(),
    );
}

#[when("按记录 ID 生成消息")]
fn run_id(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .args(["message", "demo"])
            .output()
            .unwrap(),
    );
}

#[when("将消息写入文件")]
fn run_file(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let destination = f._temp.path().join("message.txt");
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .arg("message")
            .arg("--record")
            .arg(&f.record)
            .arg("--output")
            .arg(&destination)
            .output()
            .unwrap(),
    );
    assert!(f.output.as_ref().unwrap().status.success());
    assert!(f.output.as_ref().unwrap().stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(destination).unwrap(),
        "v1.0.1\nchangelog:\n修复一\n修复二\n"
    );
}

#[when("试图用输出覆盖发布记录")]
fn overwrite(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .arg("message")
            .arg("--record")
            .arg(&f.record)
            .arg("--output")
            .arg(&f.record)
            .output()
            .unwrap(),
    );
}

#[then(expr = "消息包含 {string}")]
fn contains(world: &mut ReleaseWorld, expected: String) {
    let output = fixture(world).output.as_ref().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(&expected),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[then(expr = "错误包含 {string}")]
fn error(world: &mut ReleaseWorld, expected: String) {
    let output = fixture(world).output.as_ref().unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(&expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[then("消息只包含成功项且保持提交顺序和换行")]
fn exact(world: &mut ReleaseWorld) {
    let output = fixture(world).output.as_ref().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "v1.0.1\nchangelog:\n修复一\n修复二\n"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("v2.0.1"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("v3.0.1"));
}

#[then("发布记录和 Git 引用保持不变")]
fn unchanged(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert_eq!(std::fs::read(&f.record).unwrap(), f.original_record);
    let output = Command::new("git")
        .current_dir(&f.repo)
        .args(["tag", "--list"])
        .output()
        .unwrap();
    assert!(output.stdout.is_empty());
    let output = Command::new("git")
        .current_dir(&f.repo)
        .args(["log", "--format=%s"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&output.stdout), "initial\n");
}

#[given(expr = "发布记录 schema 为 {int}")]
fn record_schema(world: &mut ReleaseWorld, version: u32) {
    let f = fixture(world);
    let mut value: serde_json::Value = serde_json::from_slice(&f.original_record).unwrap();
    value["schema_version"] = version.into();
    std::fs::write(&f.record, serde_json::to_vec(&value).unwrap()).unwrap();
}

#[given("仓库模板使用绝对路径")]
fn absolute_template(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let template = f._temp.path().join("absolute.j2");
    std::fs::write(&template, "绝对模板|{{ releases[0].tag }}").unwrap();
    let source = toml::to_string(&serde_json::json!({
        "schema_version": 1,
        "message": {"template": template.to_str().unwrap()}
    }))
    .unwrap();
    let config = f.repo.join(".git/rls/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(config, source).unwrap();
}
