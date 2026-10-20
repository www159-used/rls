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
    output: Option<Output>,
    original_refs: String,
    original_status: String,
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

#[given("隔离的仓库、全局目录和 Git tags")]
fn setup(world: &mut ReleaseWorld) {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let global = temp.path().join("global");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&global).unwrap();
    git(&repo, &["init", "-q"]);
    std::fs::write(repo.join("user.txt"), "original").unwrap();
    git(&repo, &["add", "user.txt"]);
    git(&repo, &["commit", "-qm", "initial"]);
    git(&repo, &["tag", "v1.0.0"]);
    git(&repo, &["commit", "--allow-empty", "-qm", "修复一"]);
    git(&repo, &["commit", "--allow-empty", "-qm", "修复二"]);
    git(&repo, &["tag", "-a", "v1.0.1", "-m", "annotated release"]);
    std::fs::write(repo.join("user.txt"), "user edits").unwrap();
    let original_refs = git_text(&repo, &["show-ref"]);
    let original_status = git_text(&repo, &["status", "--porcelain"]);
    world.fixture = Some(Fixture {
        _temp: temp,
        cwd: repo.clone(),
        repo,
        global,
        output: None,
        original_refs,
        original_status,
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
            .args(["v1.0.1", "--from", "v1.0.0"])
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
            .args(["v1.0.1", "--from", "v1.0.0"])
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

#[then("消息包含区间内全部提交且保持顺序和换行")]
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
    assert!(output.stderr.is_empty());
}

#[then("工作区和 Git 引用保持不变且未生成 records")]
fn unchanged(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert_eq!(git_text(&f.repo, &["show-ref"]), f.original_refs);
    assert_eq!(
        git_text(&f.repo, &["status", "--porcelain"]),
        f.original_status
    );
    assert_eq!(
        std::fs::read_to_string(f.repo.join("user.txt")).unwrap(),
        "user edits"
    );
    assert!(!f.repo.join(".rls").exists());
}

#[given("仓库模板使用绝对路径")]
fn absolute_template(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let template = f._temp.path().join("absolute.j2");
    std::fs::write(&template, "绝对模板|{{ releases[0].tag }}").unwrap();
    let source = format!(
        "schema_version = 1\n[message]\ntemplate = {}\n",
        toml::Value::String(template.to_str().unwrap().into())
    );
    let config = f.repo.join(".git/rls/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(config, source).unwrap();
}

fn git_text(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[when(expr = "执行消息命令 {string}")]
fn custom_command(world: &mut ReleaseWorld, arguments: String) {
    let f = fixture(world);
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .args(arguments.split_whitespace())
            .output()
            .unwrap(),
    );
}

#[given(expr = "tag {string} 指向 {string}")]
fn add_tag(world: &mut ReleaseWorld, tag: String, reference: String) {
    git(&fixture(world).repo, &["tag", &tag, &reference]);
}

#[given("存在不相关分支上的同版本线 tag")]
fn unrelated(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let tree = git_text(&f.repo, &["rev-parse", "HEAD^{tree}"]);
    let output = Command::new("git")
        .current_dir(&f.repo)
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit-tree",
            tree.trim(),
            "-m",
            "unrelated",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let sha = String::from_utf8(output.stdout).unwrap();
    git(&f.repo, &["tag", "v9.0.99", sha.trim()]);
}

#[then(expr = "消息不包含 {string}")]
fn excludes(world: &mut ReleaseWorld, expected: String) {
    let output = fixture(world).output.as_ref().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&expected));
}

#[given("从深度一的浅克隆执行")]
fn shallow(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f._temp.path().join("shallow");
    git(
        &f.repo,
        &[
            "clone",
            "--depth=1",
            &format!("file://{}", f.repo.display()),
            f.cwd.to_str().unwrap(),
        ],
    );
    let source = f.repo.join(".git/rls/config.toml");
    if source.exists() {
        let destination = f.cwd.join(".git/rls/config.toml");
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::copy(source, destination).unwrap();
    }
}
