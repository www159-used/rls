mod support;

use cucumber::{World, given, then, when};
use std::path::Path;

#[derive(Debug, Default, World)]
struct ReleaseWorld {
    fixture: Option<support::Fixture>,
    source: String,
    config: Option<toml::Value>,
}

#[given("仓库中的配置示例")]
fn example_config(world: &mut ReleaseWorld) {
    world.source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("rls.example.toml"))
            .expect("配置示例应可读取");
}

#[when("按 TOML 读取示例")]
fn parse_example(world: &mut ReleaseWorld) {
    world.config = Some(toml::from_str(&world.source).expect("配置示例应为合法 TOML"));
}

#[then(expr = "分支 {string} 的版本前缀为 {string}")]
fn branch_prefix(world: &mut ReleaseWorld, branch: String, prefix: String) {
    let config = world.config.as_ref().expect("应先读取配置");
    assert_eq!(
        config["branches"][&branch]["version_prefix"].as_str(),
        Some(prefix.as_str())
    );
    assert_eq!(
        config["branches"][&branch]["tag_prefix"].as_str(),
        Some("v")
    );
}

#[then("使用 git-tag 后端且消息模板存在")]
fn backend_and_template(world: &mut ReleaseWorld) {
    let config = world.config.as_ref().expect("应先读取配置");
    assert_eq!(config["schema_version"].as_integer(), Some(1));
    assert_eq!(config["release"]["backend"].as_str(), Some("git-tag"));
    let template = config["message"]["template"]
        .as_str()
        .expect("应配置模板路径");
    assert!(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(template)
            .is_file()
    );
}

fn main() {
    futures::executor::block_on(
        ReleaseWorld::cucumber()
            .fail_on_skipped()
            .run_and_exit(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/features")),
    );
}
