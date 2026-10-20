## Why

维护者希望不同 Agent CLI 都能正确使用 rls，而不必为每个工具重复解释配置路径、消息版本和发布记录。项目需要独立的 skill 分发目录和明确的安装入口，避免把尚未实现的发布流程误当作现有功能。

## What Changes

- 提供 skills/rls，含通用 SKILL.md、格式参考和许可证原文。
- 提供 Python 标准库安装器及 make install-skill，支持通用、Codex、Claude Code、Gemini CLI、OpenCode、自定义目录和项目作用域。
- 默认拒绝覆盖已存在的 skill，显式 FORCE=1 更新。
- 按用户要求加入 GNU AGPL v3 官方原文，并设置 Cargo 的 AGPL-3.0-only 元数据。

## Capabilities

### New Capabilities
- `agent-skill`: 可移植的 rls 使用说明及跨 Agent 安装入口。

### Modified Capabilities

无。

## Impact

新增 Python 3 安装依赖，不改变 Rust 命令行为。安装 skill 与安装 rls 二进制分别执行，不修改 Agent 权限或启动配置。
