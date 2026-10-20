## Why

用户希望只编辑 Git 元目录中的模板与版本规则，消息事实由 Git 直接读取。现有 records 和 JSON 输入要求用户手工整理版本与提交，增加重复维护，也容易把手填状态误当作发布事实。

## What Changes

- **BREAKING** 移除记录 ID、--record、ReleaseRecord 和 JSON 示例。
- message 接受已有目标 tag，支持 --from 显式基线或按仓库 branches 规则自动选择。
- 读取完整 Git 提交区间，支持 annotated tag、多个目标和 worktree；拒绝无变更、无基线和歧义。
- 增加 msg/m 别名和 -c；更新说明、skill 和真实 Git 验收场景。

## Capabilities

### New Capabilities

无。

### Modified Capabilities
- `message-configuration`: Git tag 区间替代保存的发布记录作为消息来源。
- `agent-skill`: 操作说明改为直接使用 Git tags。

## Impact

不创建 records、不执行发布动作，plan/run/nexttag 仍未实现。模板的 releases 继续提供版本、tag、SHA 与 commits，增加 previous_tag/previous_sha，移除 release_id/status。branch 为匹配规则的配置键。
