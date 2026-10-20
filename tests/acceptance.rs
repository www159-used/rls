mod support;

use cucumber::{World, given, then, when};
use std::path::Path;

#[derive(Debug, Default, World)]
struct ReleaseWorld {
    fixture: Option<support::Fixture>,
    source: String,
    config: Option<toml::Value>,
}

#[given("the repository configuration example")]
fn example_config(world: &mut ReleaseWorld) {
    world.source =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("rls.example.toml"))
            .expect("configuration example must be readable");
}

#[when("parsing the example as TOML")]
fn parse_example(world: &mut ReleaseWorld) {
    world.config =
        Some(toml::from_str(&world.source).expect("configuration example must be valid TOML"));
}

#[then(expr = "branch {string} has version prefix {string}")]
fn branch_prefix(world: &mut ReleaseWorld, branch: String, prefix: String) {
    let config = world
        .config
        .as_ref()
        .expect("configuration must be loaded first");
    assert_eq!(
        config["branches"][&branch]["version_prefix"].as_str(),
        Some(prefix.as_str())
    );
    assert_eq!(
        config["branches"][&branch]["tag_prefix"].as_str(),
        Some("v")
    );
}

#[then("the git-tag backend is selected and the message template exists")]
fn backend_and_template(world: &mut ReleaseWorld) {
    let config = world
        .config
        .as_ref()
        .expect("configuration must be loaded first");
    assert_eq!(config["schema_version"].as_integer(), Some(1));
    assert_eq!(config["release"]["backend"].as_str(), Some("git-tag"));
    let template = config["message"]["template"]
        .as_str()
        .expect("template path must be configured");
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
