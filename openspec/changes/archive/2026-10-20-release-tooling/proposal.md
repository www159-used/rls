## Why

项目需要可重复的 Rust 构建工具链和统一的版本更新入口，避免开发者环境不同或手工修改 Cargo.lock 导致差异。维护者明确要求手动打 tag，因此版本工具只处理版本号与版本提交，不执行远端发布。

## What Changes

- 新增 rust-toolchain.toml 固定 1.95.0，附带 rustfmt/Clippy，并声明 rust-version。
- 新增 release.toml，启用 cargo-release 的版本管理，关闭自动 tag/push/publish。
- Makefile 提供 setup-release、release dry-run 和 release-execute。
- 文档区分本项目版本管理与尚未实现的产品 Cargo 发布后端。

## Capabilities

### New Capabilities

无。

### Modified Capabilities
- `development-workflow`: 固定工具链和手动 tag 的版本管理流程。

## Impact

本机已有 cargo-release 1.1.2，无需重新安装。不变更本次项目版本，不创建真实 tag、不推送或发布制品。
