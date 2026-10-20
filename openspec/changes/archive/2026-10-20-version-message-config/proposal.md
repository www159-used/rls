## Why

消息配置需要独立版本号，方便以后演进模板和变量语义，同时保留现有仓库和全局配置的兼容性。未知格式必须被明确拒绝，避免在继承合并后悄悄使用错误语义。

## What Changes

- 增加 message.version，当前支持整数 1；缺省兼容原始 v1。
- 各层分别校验版本，再合并消息字段。
- 更新配置示例、安装说明和验收场景。

## Capabilities

### New Capabilities

无。

### Modified Capabilities
- `message-configuration`: 消息配置独立版本与向后兼容规则。

## Impact

仅改变消息配置加载；CLI 的发布记录参数不变。安装建议使用 ~/.local，处理与 Rust 工具链同名命令的 PATH 优先级。
