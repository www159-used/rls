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

### Requirement: Git tag message source

系统 SHALL 从本地 Git tag 与提交生成消息，不读取或生成发布 records。接受一个或多个目标 tag，并支持单目标 --from。只接受指向提交的 tag，包括轻量和 annotated tag；基线 MUST 是目标祖先，且不能指向相同提交。读取基线之后到目标的全部提交并保留 SHA，使用反向拓扑顺序，不修改 Git 或未提交文件。

#### Scenario: Explicit interval
- **WHEN** 用户指定已有基线与目标 tag
- **THEN** 生成完整提交区间的消息，并保持 Git 引用和工作区内容不变

#### Scenario: Local data only
- **WHEN** tag 不在本地或输入是分支/revision 表达式
- **THEN** 命令报错，不执行 fetch 或发布动作

### Requirement: Automatic baseline by repository rule

系统 SHALL 按仓库 branches 的 tag_prefix 和 version_prefix 严格匹配数字后缀，只考虑目标祖先中的同版本线 tag，按末位数字选择上一版本。多个目标按各自规则处理，按输入顺序输出。没有基线、历史不单调、数字重复、历史分叉歧义或浅克隆 MUST 停止自动选择并提示显式基线或补全历史。

#### Scenario: Numeric ordering
- **WHEN** 同版本线含末位 2、9，目标末位为 10
- **THEN** 在祖先关系满足时选择 9，不混入其他版本线、日期 tag 或非祖先 tag

#### Scenario: Ambiguous history
- **WHEN** 候选版本顺序与祖先关系不一致
- **THEN** 不猜测基线，要求 --from

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

