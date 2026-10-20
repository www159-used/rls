## Why

维护者需要按多个分支创建递增 tag 并合并最近一次发布消息，无需手工选择 tag 区间。

## What Changes

- 将 message 位置参数改为分支列表（破坏性变更，移除 --from）。
- 增加 tag/t、counter/manual、预览、显式推送与同名重试。
- 手写 shell 补全，支持中英文界面，测试统一英文，同步 skill。

## Capabilities

### New Capabilities
- `tag-workflow`: 多分支 tag、推送和恢复。
- `cli-experience`: 别名、补全与语言。

### Modified Capabilities
- `message-configuration`: 按分支最近两个 tag 生成消息。

## Impact

Rust CLI、配置、Git 模块、Makefile、测试、README 和可移植 skill。Cargo 发布后端及持久化计划不在本次范围。
