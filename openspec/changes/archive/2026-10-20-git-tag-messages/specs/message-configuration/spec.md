## REMOVED Requirements

### Requirement: Strict rendering of saved results
**Reason**: 用户要求消息事实直接读取 Git，不维护 records。
**Migration**: 使用 message <目标tag> --from <基线tag> 或按仓库版本线自动选基线；模板不再提供 release_id/status。

### Requirement: Independent message outputs
**Reason**: 旧输出契约以记录 ID/JSON 文件及仓库外读取为前提，已移除该输入方式。
**Migration**: 使用新的 Git message outputs，在目标仓库指定 tags 生成消息。

## ADDED Requirements

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
