## ADDED Requirements

### Requirement: Portable rls skill package

项目 SHALL 提供独立的 skills/rls 目录，包含标准 SKILL.md、包内格式参考和许可证原文。说明 MUST 与已实现命令一致，区分 message.version、发布版本和记录 ID，不依赖个人路径或特定 Agent 工具。

#### Scenario: Standalone copy
- **WHEN** 安装或手动复制整个 skill 文件夹
- **THEN** SKILL.md 和所有引用文件均存在，阅读说明不再依赖源码仓库

### Requirement: Explicit installation scope

安装器 SHALL 支持通用、Codex、Claude Code、Gemini CLI 和 OpenCode 的用户级或项目级目录，并允许自定义 skills 父目录。安装仅复制 rls skill，不安装可执行程序或修改 Agent 配置。

#### Scenario: Custom path
- **WHEN** 指定包含空格的 SKILL_DIR
- **THEN** 完整 skill 安装到该目录下的 rls 子目录

#### Scenario: Project scope
- **WHEN** 指定 AGENT=claude、SCOPE=project 和 PROJECT
- **THEN** 安装到指定项目的 .claude/skills/rls

### Requirement: Existing installation protection

安装器 SHALL 默认拒绝覆盖已有目标。只有显式 --force 或 FORCE=1 才替换 rls 子目录，且不改变相邻 skill。目标符号链接和源码重叠路径 MUST 被拒绝。

#### Scenario: Existing edits
- **WHEN** 目标含本地编辑且未指定 force
- **THEN** 安装失败并保留现有内容

#### Scenario: Explicit update
- **WHEN** 对普通目录指定 force
- **THEN** 更新全部分发文件、移除旧文件并保留相邻目录
