# message-configuration Specification

## Purpose
定义仓库与全局消息配置的发现、合并和模板渲染行为，使维护者可从已有 Git tag 区间重复生成文案，无需维护 records 或重新发布。
## Requirements
### Requirement: Layered message configuration

系统 SHALL 从共享 Git 元目录的 rls/config.toml 和全局用户配置加载消息设置，支持 RLS_CONFIG_HOME。仓库模板优先于全局和内置模板，变量按键覆盖且保留空字符串；模板相对路径基于声明目录。

#### Scenario: Inherit and override
- **WHEN** 仓库仅覆盖全局配置的 greeting
- **THEN** 保留全局模板来源和其他变量，使用仓库 greeting

#### Scenario: Worktree discovery
- **WHEN** 从 linked worktree 或仓库子目录生成消息
- **THEN** 使用共享 Git 元目录下的仓库配置

### Requirement: Versioned message configuration

系统 SHALL 支持独立的 message.version 整数标记，当前格式为 1，省略时按 v1 读取。每个配置层 MUST 在合并前分别校验版本；不支持的版本报错并指出源文件，不修改配置。此字段与顶层 schema_version 和发布版本相互独立。

#### Scenario: Legacy compatibility
- **WHEN** 仓库省略 message.version，全局声明 version = 1
- **THEN** 两层均按 v1 解释并按既有规则合并

#### Scenario: Unknown version cannot be overridden
- **WHEN** 全局或仓库任一层使用不支持的消息版本
- **THEN** 返回非零状态并显示该配置路径与版本，不因其他层声明 v1 而忽略错误

#### Scenario: Invalid version type
- **WHEN** message.version 为字符串
- **THEN** 配置解析失败并指出来源

### Requirement: Branch message source

系统 SHALL 在 tag_pattern 存在时改用严格祖先链选择最近两个 tag；msg 不执行命名脚本。version SHALL 仅按所选分支的 tag_prefix 去前缀，不跨分支推断。

系统 SHALL 接受分支列表，默认当前分支；message/msg/m 等价。每个分支按仓库规则选取可达的最近两个 tag，数字规则按数值排序，pattern 规则按祖先链排序，读取其间全部提交并按参数顺序一次渲染。只读取本地 Git，不接受 tag 参数、range 或 --from，不保存 records。

#### Scenario: Multiple branches
- **WHEN** 用户选择两个有至少两个版本 tag 的分支
- **THEN** 合并各分支最近一次发布消息，不包含最新 tag 后的提交

#### Scenario: Invalid selection
- **WHEN** 任一分支缺失规则、少于两个 tag、存在数字歧义、非单调历史或浅克隆
- **THEN** 命令失败且不输出部分消息

#### Scenario: Detached checkout
- **WHEN** detached HEAD 下省略分支
- **THEN** 要求显式指定分支

### Requirement: Git message outputs

系统 SHALL 将 Git 区间与配置渲染成文本，输出终端、文件或剪贴板。支持 message、msg、m，以及 --output/-o、--copy/-c。模板严格检查未定义变量，保留提交顺序与空行，配置或渲染失败不输出半条消息。消息输出失败不触发发布操作。

#### Scenario: File output
- **WHEN** 指定 --output
- **THEN** 完整文本写入文件并保留换行，不创建 tag 或 records

#### Scenario: Regenerate after editing metadata
- **WHEN** 修改元目录消息变量后重新运行相同 tag 区间
- **THEN** 新文案生效且 Git 引用不变

#### Scenario: Strict template errors
- **WHEN** 模板缺失、语法错误或引用未定义变量
- **THEN** 命令以非零状态退出并指出来源

