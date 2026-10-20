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

#[given("an isolated repository, global directory and Git tags")]
fn setup(world: &mut ReleaseWorld) {
    let temp = tempfile::tempdir().unwrap();
    let repo = temp.path().join("repo");
    let global = temp.path().join("global");
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&global).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    std::fs::write(repo.join("user.txt"), "original").unwrap();
    git(&repo, &["add", "user.txt"]);
    git(&repo, &["commit", "-qm", "initial"]);
    git(&repo, &["tag", "v1.0.0"]);
    git(&repo, &["commit", "--allow-empty", "-qm", "First fix"]);
    git(&repo, &["commit", "--allow-empty", "-qm", "Second fix"]);
    git(&repo, &["tag", "-a", "v1.0.1", "-m", "annotated release"]);
    std::fs::write(repo.join("user.txt"), "user edits").unwrap();
    std::fs::create_dir_all(repo.join(".git/rls")).unwrap();
    std::fs::write(
        repo.join(".git/rls/config.toml"),
        "schema_version = 1\n[branches.main]\nversion_prefix = \"1.0\"\ntag_prefix = \"v\"\n",
    )
    .unwrap();
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

#[given(expr = "the {word} configuration is")]
fn config(world: &mut ReleaseWorld, layer: String, step: &Step) {
    let f = fixture(world);
    let path = match layer.as_str() {
        "global" => f.global.join("config.toml"),
        "repository" => f.repo.join(".git/rls/config.toml"),
        _ => panic!("unknown layer"),
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut source = step.docstring.as_ref().unwrap().clone();
    if layer == "repository" && !source.contains("[branches.") {
        source.push_str("\n[branches.main]\nversion_prefix = \"1.0\"\ntag_prefix = \"v\"\n");
    }
    std::fs::write(path, source).unwrap();
}

#[given(expr = "the {word} template is")]
fn template(world: &mut ReleaseWorld, layer: String, step: &Step) {
    let f = fixture(world);
    let root = match layer.as_str() {
        "global" => f.global.clone(),
        "repository" => f.repo.join(".git/rls"),
        _ => panic!("unknown layer"),
    };
    std::fs::create_dir_all(root.join("templates")).unwrap();
    std::fs::write(
        root.join("templates/message.j2"),
        step.docstring.as_ref().unwrap(),
    )
    .unwrap();
}

#[given("running from a repository subdirectory")]
fn subdir(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f.repo.join("nested/child");
    std::fs::create_dir_all(&f.cwd).unwrap();
}

#[given("running from a linked worktree")]
fn worktree(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f._temp.path().join("worktree");
    git(
        &f.repo,
        &["worktree", "add", "--detach", f.cwd.to_str().unwrap()],
    );
}

#[given("running outside the repository")]
fn outside(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.cwd = f._temp.path().into();
}

#[when("generating the message")]
fn run(world: &mut ReleaseWorld) {
    let f = fixture(world);
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .env("RLS_LANG", "en")
            .arg("message")
            .arg("main")
            .output()
            .unwrap(),
    );
}

#[when("writing the message to a file")]
fn run_file(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let destination = f._temp.path().join("message.txt");
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .env("RLS_LANG", "en")
            .arg("message")
            .arg("main")
            .arg("--output")
            .arg(&destination)
            .output()
            .unwrap(),
    );
    assert!(f.output.as_ref().unwrap().status.success());
    assert!(f.output.as_ref().unwrap().stdout.is_empty());
    assert_eq!(
        std::fs::read_to_string(destination).unwrap(),
        "v1.0.1\nchangelog:\nFirst fix\nSecond fix\n"
    );
}

#[then(expr = "the message contains {string}")]
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

#[then(expr = "the error contains {string}")]
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

#[then("the message contains all commits in order with preserved newlines")]
fn exact(world: &mut ReleaseWorld) {
    let output = fixture(world).output.as_ref().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "v1.0.1\nchangelog:\nFirst fix\nSecond fix\n"
    );
    assert!(output.stderr.is_empty());
}

#[then("the worktree and Git refs are unchanged and no records are created")]
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

#[given("the repository template uses an absolute path")]
fn absolute_template(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let template = f._temp.path().join("absolute.j2");
    std::fs::write(&template, "Absolute template|{{ releases[0].tag }}").unwrap();
    let source = format!(
        "schema_version = 1\n[message]\ntemplate = {}\n",
        toml::Value::String(template.to_str().unwrap().into())
    );
    let config = f.repo.join(".git/rls/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(
        config,
        format!("{source}\n[branches.main]\nversion_prefix = \"1.0\"\ntag_prefix = \"v\"\n"),
    )
    .unwrap();
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

#[when(expr = "running the CLI with {string}")]
fn custom_command(world: &mut ReleaseWorld, arguments: String) {
    let f = fixture(world);
    f.output = Some(
        Command::new(env!("CARGO_BIN_EXE_rls"))
            .current_dir(&f.cwd)
            .env("RLS_CONFIG_HOME", &f.global)
            .env("RLS_LANG", "en")
            .args(arguments.split_whitespace())
            .output()
            .unwrap(),
    );
}

#[given(expr = "tag {string} points to {string}")]
fn add_tag(world: &mut ReleaseWorld, tag: String, reference: String) {
    git(&fixture(world).repo, &["tag", &tag, &reference]);
}

#[given("a matching tag exists on unrelated history")]
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

#[then(expr = "the message excludes {string}")]
fn excludes(world: &mut ReleaseWorld, expected: String) {
    let output = fixture(world).output.as_ref().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains(&expected));
}

#[given("running from a depth-one shallow clone")]
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

#[given("a local bare remote and two release branches exist")]
fn remote_fixture(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let remote = f._temp.path().join("remote.git");
    git(&f.repo, &["init", "--bare", remote.to_str().unwrap()]);
    git(
        &f.repo,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    git(&f.repo, &["branch", "maintenance"]);
    git(&f.repo, &["tag", "v2.0.0", "HEAD~2"]);
    git(&f.repo, &["tag", "v2.0.1", "HEAD"]);
    git(
        &f.repo,
        &["commit", "--allow-empty", "-qm", "Pending release"],
    );
    git(&f.repo, &["branch", "-f", "maintenance", "HEAD"]);
    git(
        &f.repo,
        &["push", "origin", "main", "maintenance", "--tags"],
    );
    git(&f.repo, &["commit", "--allow-empty", "-qm", "Local only"]);
    std::fs::write(f.repo.join(".git/rls/config.toml"), "schema_version=1\n[branches.main]\nversion_prefix='1.0'\ntag_prefix='v'\n[branches.maintenance]\nversion_prefix='2.0'\ntag_prefix='v'\n").unwrap();
}

#[then("both new tags point to remote commits and are pushed without changing the worktree")]
fn pushed(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(
        f.output.as_ref().unwrap().status.success(),
        "{:?}",
        f.output
    );
    let target = git_text(&f.repo, &["rev-parse", "HEAD~1"]);
    for tag in ["v1.0.2", "v2.0.2"] {
        assert_eq!(git_text(&f.repo, &["rev-parse", tag]), target);
        assert!(
            git_text(&f.repo, &["ls-remote", "--tags", "origin", tag]).starts_with(target.trim())
        );
    }
    assert_eq!(
        std::fs::read_to_string(f.repo.join("user.txt")).unwrap(),
        "user edits"
    );
}

#[then("no new tags are created")]
fn no_new_tags(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(f.output.as_ref().unwrap().status.success());
    assert!(!git_text(&f.repo, &["tag", "--list"]).contains("v1.0.2"));
    assert!(!git_text(&f.repo, &["ls-remote", "--tags", "origin"]).contains("v1.0.2"));
}

#[then("only a local tag is created and pushing the same name can be retried")]
fn retry_push(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(f.output.as_ref().unwrap().status.success());
    assert!(git_text(&f.repo, &["ls-remote", "--tags", "origin", "v1.0.2"]).is_empty());
    custom_command(world, "t main --name v1.0.2 -p".into());
    let f = fixture(world);
    assert!(
        f.output.as_ref().unwrap().status.success(),
        "{:?}",
        f.output
    );
    assert!(!git_text(&f.repo, &["ls-remote", "--tags", "origin", "v1.0.2"]).is_empty());
    assert!(!git_text(&f.repo, &["tag", "--list"]).contains("v1.0.3"));
}

#[given("main uses manual increment mode")]
fn manual(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let path = f.repo.join(".git/rls/config.toml");
    let source = std::fs::read_to_string(&path)
        .unwrap()
        .replace("[branches.main]", "[branches.main]\nincrement='manual'");
    std::fs::write(path, source).unwrap();
}

#[then("the valid branch succeeds and the batch fails while retaining results")]
fn partial(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let output = f.output.as_ref().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("v1.0.2"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("branches.missing"));
    assert_eq!(
        git_text(&f.repo, &["rev-parse", "v1.0.2"]),
        git_text(&f.repo, &["rev-parse", "HEAD~1"])
    );
}

#[given("the remote temporarily rejects pushes")]
fn reject_push(world: &mut ReleaseWorld) {
    use std::os::unix::fs::PermissionsExt;
    let path = fixture(world)
        ._temp
        .path()
        .join("remote.git/hooks/pre-receive");
    std::fs::write(&path, "#!/bin/sh\nexit 1\n").unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[then("the rejected push retains its tag and can be retried after recovery")]
fn failed_push_retry(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(!f.output.as_ref().unwrap().status.success());
    assert!(String::from_utf8_lossy(&f.output.as_ref().unwrap().stderr).contains("--name v1.0.2"));
    assert_eq!(
        git_text(&f.repo, &["rev-parse", "v1.0.2"]),
        git_text(&f.repo, &["rev-parse", "HEAD~1"])
    );
    std::fs::remove_file(f._temp.path().join("remote.git/hooks/pre-receive")).unwrap();
    custom_command(world, "t main --name v1.0.2 -p".into());
    let f = fixture(world);
    assert!(f.output.as_ref().unwrap().status.success());
    assert!(!git_text(&f.repo, &["ls-remote", "--tags", "origin", "v1.0.2"]).is_empty());
}

#[given("script-based release branches")]
fn script_branches(world: &mut ReleaseWorld) {
    remote_fixture(world);
    let f = fixture(world);
    git(&f.repo, &["tag", "release-main-z", "HEAD~3"]);
    git(&f.repo, &["tag", "release-main-a", "HEAD~2"]);
    git(&f.repo, &["tag", "release-maintenance-z", "HEAD~3"]);
    git(&f.repo, &["tag", "release-maintenance-a", "HEAD~2"]);
    let source = ["main", "maintenance"].map(|branch| format!(
        "[branches.{branch}]\nincrement='script'\ntag_pattern='release-{branch}-*'\ntag_prefix='release-'\n[branches.{branch}.script]\ncommand=['python3','./naming.py']\ntimeout_seconds=1\n"
    )).join("");
    std::fs::write(
        f.repo.join(".git/rls/config.toml"),
        format!("schema_version=1\n{source}"),
    )
    .unwrap();
}

#[given("the naming script is")]
fn naming_script(world: &mut ReleaseWorld, step: &Step) {
    std::fs::write(
        fixture(world).repo.join("naming.py"),
        step.docstring.as_ref().unwrap(),
    )
    .unwrap();
}

#[then("custom tags are pushed at the remote commits")]
fn custom_tags_pushed(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(
        f.output.as_ref().unwrap().status.success(),
        "{:?}",
        f.output
    );
    for branch in ["main", "maintenance"] {
        let tag = format!("release-{branch}-next");
        let target = git_text(&f.repo, &["rev-parse", "HEAD~1"]);
        assert_eq!(git_text(&f.repo, &["rev-parse", &tag]), target);
        assert!(
            git_text(&f.repo, &["ls-remote", "--tags", "origin", &tag]).starts_with(target.trim())
        );
    }
    assert_eq!(
        std::fs::read_to_string(f.repo.join("user.txt")).unwrap(),
        "user edits"
    );
}

#[then("no custom release tag exists")]
fn no_custom_tag(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(!git_text(&f.repo, &["tag", "--list"]).contains("-next"));
    assert!(!git_text(&f.repo, &["ls-remote", "--tags", "origin"]).contains("-next"));
}

#[then("the same custom tag can be pushed without running the script")]
fn custom_retry(world: &mut ReleaseWorld) {
    let f = fixture(world);
    assert!(!f.output.as_ref().unwrap().status.success());
    assert!(
        String::from_utf8_lossy(&f.output.as_ref().unwrap().stderr)
            .contains("--name release-main-next")
    );
    std::fs::remove_file(f._temp.path().join("remote.git/hooks/pre-receive")).unwrap();
    std::fs::write(
        f.repo.join("naming.py"),
        "raise Exception('must not execute')",
    )
    .unwrap();
    custom_command(world, "t main --name release-main-next -p".into());
    let f = fixture(world);
    assert!(
        f.output.as_ref().unwrap().status.success(),
        "{:?}",
        f.output
    );
    assert!(
        !git_text(
            &f.repo,
            &["ls-remote", "--tags", "origin", "release-main-next"]
        )
        .is_empty()
    );
}

#[given("no custom baseline tags exist")]
fn remove_custom_tags(world: &mut ReleaseWorld) {
    let f = fixture(world);
    for tag in ["release-main-z", "release-main-a"] {
        git(&f.repo, &["tag", "-d", tag]);
    }
}

#[given("custom tag history forks before the target")]
fn forked_custom_history(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let tree = git_text(&f.repo, &["rev-parse", "HEAD^{tree}"]);
    let base = git_text(&f.repo, &["rev-parse", "HEAD~3"]);
    let first = git_text(&f.repo, &["rev-parse", "HEAD~2"]);
    let commit_tree = |parents: &[&str], message: &str| {
        let mut command = Command::new("git");
        command.current_dir(&f.repo).args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit-tree",
            tree.trim(),
        ]);
        for parent in parents {
            command.args(["-p", parent]);
        }
        let out = command.args(["-m", message]).output().unwrap();
        assert!(out.status.success());
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    };
    let sibling = commit_tree(&[base.trim()], "sibling");
    let merge = commit_tree(&[first.trim(), &sibling], "merge");
    git(&f.repo, &["tag", "release-main-sibling", &sibling]);
    git(&f.repo, &["update-ref", "refs/heads/main", &merge]);
}

#[given("the message shows the selected branch version")]
fn branch_version_template(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let path = f.repo.join(".git/rls/config.toml");
    let mut source = std::fs::read_to_string(&path).unwrap();
    source.push_str("\n[message]\ntemplate='version.j2'\n");
    std::fs::write(path, source).unwrap();
    std::fs::write(
        f.repo.join(".git/rls/version.j2"),
        "{{ releases[0].branch }}|{{ releases[0].version }}|{{ releases[0].previous_tag }}",
    )
    .unwrap();
}

#[given("the next custom name is excluded")]
fn excluded_custom_name(world: &mut ReleaseWorld) {
    let f = fixture(world);
    let path = f.repo.join(".git/rls/config.toml");
    let source = std::fs::read_to_string(&path).unwrap().replace(
        "[branches.main]",
        "[branches.main]\nexclude_tags=['release-main-next']",
    );
    std::fs::write(path, source).unwrap();
}
