## MODIFIED Requirements

### Requirement: Portable rls skill package

项目 SHALL 提供独立的 skills/rls 目录，包含标准 SKILL.md、包内格式参考和许可证原文。说明 MUST 与已实现命令一致，区分 message.version 与发布版本，指导从 Git tag 区间生成消息，明确不需要 records，不依赖个人路径或特定 Agent 工具。

#### Scenario: Standalone copy
- **WHEN** 安装或手动复制整个 skill 文件夹
- **THEN** SKILL.md 和所有引用文件均存在，阅读说明不再依赖源码仓库
