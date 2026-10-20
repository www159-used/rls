## ADDED Requirements

### Requirement: Multi-branch tagging

系统 SHALL 基于远端分支创建递增 tag，支持 tag/t、counter/manual、--name、--dry-run/-n、--push/-p、--remote/-r。默认不推送；不覆盖冲突，不修改工作区。部分失败保留成功项并退出非零。

#### Scenario: Remote source
- **WHEN** 本地领先远端且执行多分支 tag --push
- **THEN** tags 指向远端提交，仅推送所选 tags

#### Scenario: Retry rejected push
- **WHEN** 推送失败后以相同 --name 重试且目标 SHA 不变
- **THEN** 复用本地 tag 并补推，不再次递增

#### Scenario: Preview
- **WHEN** 使用 --dry-run
- **THEN** 可 fetch 引用，但不创建发布 tag 或推送
