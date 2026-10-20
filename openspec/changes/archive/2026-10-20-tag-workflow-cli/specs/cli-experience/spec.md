## ADDED Requirements

### Requirement: Portable CLI experience

系统 SHALL 提供显式别名、项目手写 Bash/Zsh/Fish 补全及 en、zh-CN 界面。语言优先级为 --lang、RLS_LANG、LC_ALL、LC_MESSAGES、LANG；不翻译用户模板和 Git 数据。测试描述与常规数据 SHALL 使用英文。

#### Scenario: Explicit language
- **WHEN** 中文环境指定 --lang en
- **THEN** 项目帮助、结果和错误使用英文

#### Scenario: Completion
- **WHEN** 补全分支参数
- **THEN** 返回配置分支，且不 fetch 或执行发布
