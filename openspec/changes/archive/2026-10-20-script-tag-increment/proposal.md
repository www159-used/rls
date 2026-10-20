## Why

目前 manual 仅要求 --name，不能通过外部脚本接入项目自己的版本规则。维护者需要稳定、带版本的进程协议，让脚本决定 tag 名称，同时仍由 rls 校验 Git 状态、创建 tag 和推送，并让消息命令识别自定义命名。

## What Changes

- 新增 increment = "script" 和版本化 JSON stdin/stdout 协议。
- 将 tag 名称选择与历史筛选、排序解耦，有 tag_pattern 即按祖先链选择，不新增 tag_order。
- 保留 counter 与现有 manual，--name 可绕过脚本用于指定名称和恢复。
- 脚本路径统一基于当前 worktree；首版仅支持 Unix 脚本执行，Windows 明确拒绝调用。
- 同步英文验收、i18n、帮助及通用 skill。

## Capabilities

### New Capabilities
- `script-tag-increment`: 脚本命名协议、进程生命周期和自定义历史选择。

### Modified Capabilities
- `tag-workflow`: 命名阶段可调用脚本，继续沿用显式推送和恢复语义。
- `message-configuration`: 根据独立历史规则选择最近两个 tag。

## Impact

config、Git 历史选择、tag 执行模块及新增 script runner。不将脚本用于消息渲染，不新增发布 records，不允许脚本输出改变目标 SHA 或推送设置。
