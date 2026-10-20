## ADDED Requirements

### Requirement: Layered message configuration

系统 SHALL 从共享 Git 元目录的 rls/config.toml 和全局用户配置加载消息设置，支持 RLS_CONFIG_HOME。仓库模板优先于全局和内置模板，变量按键覆盖且保留空字符串；模板相对路径基于声明目录。

#### Scenario: Inherit and override
- **WHEN** 仓库仅覆盖全局配置的 greeting
- **THEN** 保留全局模板来源和其他变量，使用仓库 greeting

#### Scenario: Worktree discovery
- **WHEN** 从 linked worktree 或仓库子目录生成消息
- **THEN** 使用共享 Git 元目录下的仓库配置

### Requirement: Strict rendering of saved results

系统 SHALL 只向模板提供记录中的成功发布项及独立 variables，保留提交顺序和模板空行。配置错误、未知 schema、显式模板缺失、语法错误和未定义变量 MUST 使命令失败并指出来源。

#### Scenario: Render without publishing
- **WHEN** 同一记录在修改消息变量后重新生成
- **THEN** 文案更新且记录和 Git 引用不变，失败及跳过项仅单独报告

#### Scenario: Invalid input
- **WHEN** 显式模板缺失或变量未定义
- **THEN** 返回非零退出状态且不静默回退

### Requirement: Independent message outputs

系统 SHALL 支持按 ID 或显式 JSON 路径读取版本化发布记录，并输出终端、文件或剪贴板。输出不得覆盖输入记录。

#### Scenario: Explicit record outside a repository
- **WHEN** 仓库外传入 --record
- **THEN** 使用全局或内置设置生成消息

#### Scenario: File output
- **WHEN** 指定 --output
- **THEN** 将完整文本写入文件并保留换行，不修改记录
