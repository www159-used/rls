## Why

消息需要按用户和仓库定制，现有代码只有配置示例测试，无法加载设置或渲染发布记录。维护者需要在多个仓库之间复用通用模板，同时为各项目单独调整称呼和格式。

## What Changes

- 新增共享 Git 元目录及全局配置发现、消息字段合并和严格模板渲染。
- 提供 `rls message <release-id>` 和 `--record <JSON>`，支持终端、文件与剪贴板输出。
- 新增发布记录输入格式和示例，使用真实临时 Git 仓库覆盖配置及 worktree 行为。

## Capabilities

### New Capabilities
- `message-configuration`: 从分层配置和已有发布记录生成消息。

### Modified Capabilities

无。

## Impact

新增 Rust 库与 CLI，使用 MiniJinja、Serde、clap、directories 和 arboard。不实现 plan/run 或发布后端；未来 planner 指纹必须排除消息设置。
