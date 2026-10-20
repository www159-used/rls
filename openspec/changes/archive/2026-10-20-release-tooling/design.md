## Context

既有代码已在 Rust 1.95.0 验证。Cargo publish=false 禁止注册表发布，同时 cargo-release 默认可能跳过该包，因此要单独启用 release=true。

## Goals / Non-Goals

固定工具链、管理版本提交，维护者手动打 tag。不执行本项目发布，不实现 rls 的 Cargo 后端。

## Decisions

使用 rust-toolchain.toml 的精确版本、minimal profile 和 rustfmt/Clippy。Cargo rust-version 声明已验证的最低支持基线 1.95，不承诺更早版本。

release.toml 设置 release=true、publish=false、tag=false、push=false，版本提交消息为 chore: release {{version}}。Makefile 指定 --isolated --config release.toml，避免用户全局设置介入；默认 LEVEL=patch，用户可选其他级别或显式版本。execute 入口保留交互确认。

## Validation

临时仓库验证默认 dry-run 不改文件或 HEAD；执行路径仅在隔离仓库测试，检查 Cargo.toml/Cargo.lock、版本提交及空 tag 列表。实际工作区运行固定工具链的测试和检查。
