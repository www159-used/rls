## ADDED Requirements

### Requirement: Versioned script naming protocol

系统 SHALL 支持仓库层 increment=script，通过参数数组启动程序并用单份 UTF-8 JSON stdin/stdout 交换 version=1 的输入和 tag/skip 响应。非零退出、未知版本、重复或未知字段、错误类型、额外输出 MUST 失败；脚本不得通过响应改变 Git 目标或推送行为。

#### Scenario: Initial release
- **WHEN** 首次发布没有匹配 tag 且脚本返回合法名称
- **THEN** 输入 previous 字段为 null，rls 校验后按固定远端 SHA 创建 tag

#### Scenario: Skip
- **WHEN** 脚本成功返回 skip 与非空 reason
- **THEN** 报告跳过原因，不创建 tag，该分支不视为失败；所有分支均 skip 或无变更且无失败时退出码为 0

#### Scenario: Invalid response
- **WHEN** 脚本返回未知协议版本或 stdout 混入日志
- **THEN** 该分支失败，rls 不创建或推送其 tag

### Requirement: Bounded script execution

系统 SHALL 在 Unix 支持脚本执行，Windows 上需要调用脚本的分支 MUST 在 fetch 前明确报不支持，不启动脚本；msg 与 --name 不执行脚本，不受此限制。系统 SHALL 提供正整数秒超时，默认 10 秒，并发读取 stdout/stderr，限制 stdout 为 64 KiB、保留 stderr 尾部 64 KiB。超时或 stdout 超限 MUST 终止 Unix 独立进程组、回收直接子进程并关闭管道；脚本不得主动脱离进程组。dry-run SHALL 调用脚本并传 dry_run=true，不承诺脚本副作用隔离。

#### Scenario: Hanging child
- **WHEN** 脚本的后代保持输出管道打开直到超时
- **THEN** runner 终止该 Unix 进程组并有界退出，报告分支失败

#### Scenario: Preview
- **WHEN** 用户执行 tag --dry-run
- **THEN** 脚本收到 dry_run=true，rls 只预览经校验的名称

### Requirement: Independent tag history

系统 SHALL 在配置 tag_pattern 时使用祖先链排序，不提供 tag_order；script 必须配置非空 pattern，counter 禁止 pattern，manual 可选。有 pattern 时 version_prefix MUST 拒绝，无 pattern 时 version_prefix/tag_prefix MUST 保持原数字规则。完整名称匹配后先应用 exclude_tags，再按目标可达提交的严格祖先链选择最近 tag。msg SHALL 使用最近两个 tag 且不执行脚本。同提交多 tag、分叉与浅克隆 MUST 报错。无 pattern 时继续沿用现有数字规则。

#### Scenario: Custom names
- **WHEN** 匹配的 tag 名不含数字版本但提交形成严格祖先链
- **THEN** msg 按链中最近两个 tag 渲染，与名称字典序无关

#### Scenario: Ambiguous history
- **WHEN** 两个匹配 tag 不可比较或指向同一提交
- **THEN** 拒绝猜测最新 tag，提示历史歧义

### Requirement: Explicit retry bypass

系统 SHALL 在指定 --name 时绕过脚本，沿用名称、排除项及 SHA 校验。相同名称和 SHA 可复用补推，不同 SHA 不得覆盖；批量部分失败保留其他成功项。

#### Scenario: Retry push
- **WHEN** 脚本生成 tag 后推送失败，用户按原 --name 重试
- **THEN** 不重新调用脚本，不递增，复用原 tag 并按显式 push 参数执行


### Requirement: Worktree execution context

系统 SHALL 将 cwd 固定为当前 worktree 根目录，裸可执行名称走 PATH，带相对路径的可执行文件相对 worktree 根解析，其他参数原样传递。脚本来自当前工作区而非目标提交，不创建临时检出；协议输入 SHALL 只含 version、branch、target_sha、previous_tag、previous_sha、dry_run，不含 remote。

#### Scenario: Script differs from release commit
- **WHEN** 当前 worktree 的脚本或版本文件与目标提交不同
- **THEN** 执行当前脚本并提供固定 target_sha，脚本可通过 git show 读取发布文件

### Requirement: Pattern validation and template version

系统 SHALL 校验新 tag 匹配当前分支 pattern、未被 exclude_tags 排除、ref 合法且未被不同 SHA 占用。存在基线时新 tag 的目标 MUST 是其严格后代；相同名称和 SHA 的已有 tag 重试不要求成为自身后代。模板 version SHALL 仅按所选分支 tag_prefix 去掉一次前缀，省略或不匹配则保持完整 tag 名，不使用跨分支数字规则推断。

#### Scenario: Name outside selected pattern
- **WHEN** 脚本返回合法 Git 名称但不匹配所选分支 pattern
- **THEN** 该分支失败且不创建或推送 tag

#### Scenario: Custom version prefix
- **WHEN** 分支 tag_prefix 为 release-，最近 tag 为 release-5.6-build-a
- **THEN** 消息 version 为 5.6-build-a，branch 为用户选择的分支

#### Scenario: Unsupported execution platform
- **WHEN** Windows 上运行需要调用脚本的 tag 分支
- **THEN** 在该分支 fetch 前报告不支持，不启动脚本，也不创建或推送其 tag
