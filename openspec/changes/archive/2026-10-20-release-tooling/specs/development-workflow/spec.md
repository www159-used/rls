## ADDED Requirements

### Requirement: Pinned Rust toolchain

项目 SHALL 固定 Rust 1.95.0 工具链，包含 rustfmt 与 Clippy，并在 Cargo.toml 声明 rust-version = 1.95。

#### Scenario: Repository toolchain selection
- **WHEN** 在仓库中运行 rustup show active-toolchain
- **THEN** 选择 rust-toolchain.toml 指定的 1.95.0

### Requirement: Version management with manual tags

项目 SHALL 提供 cargo-release 1.1.2 安装入口和隔离的项目发布配置，管理版本与版本提交。自动 tag、push 和 crates.io publish MUST 关闭，tag 由维护者手动创建。

#### Scenario: Preview by default
- **WHEN** 在干净仓库执行 make release LEVEL=patch
- **THEN** 预览版本变化而不修改 Cargo 文件、HEAD 或 tags

#### Scenario: Explicit execution
- **WHEN** 维护者运行 make release-execute 并确认
- **THEN** 更新 Cargo.toml/Cargo.lock 并创建版本提交，不创建 tag、不推送、不发布制品
