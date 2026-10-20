# tag-workflow Specification

## Purpose

定义 tag-workflow 的可观察 CLI 行为，确保维护者可以在多个分支上获得一致、可验证且可重试的操作体验，并明确本地 Git 状态、远端推送以及用户配置之间的边界。
## Requirements
### Requirement: Multi-branch tagging

系统 SHALL 额外支持 script 模式，以版本化 JSON 调用 Unix 命名脚本；脚本仅返回名称或 skip，Git 目标、冲突检查与推送仍由 rls 控制。--name SHALL 绕过脚本用于恢复。

系统 SHALL 基于远端分支创建递增 tag，支持 tag/t、counter/manual/script、--name、--dry-run/-n、--push/-p、--remote/-r。默认不推送；不覆盖冲突，不修改工作区。部分失败保留成功项并退出非零。

#### Scenario: Remote source
- **WHEN** 本地领先远端且执行多分支 tag --push
- **THEN** tags 指向远端提交，仅推送所选 tags

#### Scenario: Retry rejected push
- **WHEN** 推送失败后以相同 --name 重试且目标 SHA 不变
- **THEN** 复用本地 tag 并补推，不再次递增

#### Scenario: Preview
- **WHEN** 使用 --dry-run
- **THEN** 可 fetch 引用，但不创建发布 tag 或推送

### Requirement: Explicit legacy tag exclusions

系统 SHALL 支持仓库分支配置 exclude_tags，默认为空，按完整 tag 名精确排除已确认属于旧版本线的特殊编号。排除规则同时用于 tag 递增、消息区间选择及 --name 校验，不删除或移动任何 Git tag，不忽略 fetch 冲突。未明确排除的历史仍须通过现有歧义检查。

#### Scenario: Legacy numbering
- **WHEN** v1.0.1007 是旧提交且配置排除它，正式版本最新为 v1.0.1
- **THEN** 下一 tag 为 v1.0.2，消息使用正式版本区间，旧 tag 保持原样
