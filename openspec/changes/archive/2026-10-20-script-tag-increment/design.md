## Context

现有 counter 按数字递增，manual 要求 --name。新需求是通过 shell 脚本自行计算名称。现有未提交的 release push、exclude_tags 和模板改动保留，本设计不改其行为。

## Goals / Non-Goals

提供可重试、可诊断的脚本命名接口，同时支持 msg 识别自定义 tag。脚本不负责 Git 发布，不引入 shell 命令字符串拼接、交互终端或脚本执行沙箱。

## Decisions

### Configuration and migration

新增 script 模式；保留 manual 原语义，避免已有配置突然开始执行程序。旧配置无需变更。

```toml
[branches."5.6"]
increment = "script"
tag_pattern = "release-5.6-*"
tag_prefix = "release-"
exclude_tags = []

[branches."5.6".script]
command = ["bash", "./scripts/next-tag.sh"]
timeout_seconds = 10
```

command 是非空参数数组，不进行 eval、变量展开或字符串拼接。cwd 固定为当前 worktree 根目录；command[0] 为裸程序名时通过 PATH 查找，带路径分隔符时相对于 worktree 根解析，绝对路径保持不变。其余参数原样传入，由被调用程序解释。所有相对文件路径因此只使用 worktree 根这一套基准，不再以配置目录解析可执行文件。

只执行仓库层显式配置，不继承全局脚本。缺少 command 或 timeout_seconds 非正整数时报错。不提供 tag_order；配置组合固定为：

| increment | tag_pattern | 历史规则与必要字段 |
|---|---|---|
| counter（默认） | 禁止 | 保留 version_prefix/tag_prefix 必填的数字规则 |
| script | 必填且非空 | 按匹配 tag 的祖先链；version_prefix 不参与，配置它时报错 |
| manual | 有 | 与 script 共用祖先链；version_prefix 不参与，配置它时报错 |
| manual | 无 | 保留原 version_prefix/tag_prefix 数字规则 |

祖先链模式下 tag_prefix 可选，仅用于当前分支的模板 version 去前缀；省略或名称不含此前缀时保留完整名称。tag_pattern 才负责筛选历史。非 script 模式配置 script 子表时报错，避免忽略看似生效的命名配置。旧配置无需修改。

### History selection

tag_pattern 为大小写敏感、匹配整个 tag 名的 glob；首版只支持 * 和 ?，它们可匹配 /，不支持字符组、正则或隐式前缀。exclude_tags 仍按完整名字优先排除，包括 --name 和脚本结果。

未配置 tag_pattern 时沿用现有严格数字与祖先关系检查。配置 tag_pattern 时只考虑目标提交可达、指向提交的匹配 tag，以祖先关系形成严格链；后代比祖先新。最近两个链节点用于 msg，最近一个用于脚本 previous 字段。不按提交时间、tag 时间或字典序排序。同提交多个 tag 或不可比较的分叉时报歧义，不自动挑选。浅克隆继续拒绝自动选择。

首次无匹配 tag 时 tag 可调用脚本，previous_tag/previous_sha 均为 null；msg 仍要求至少两个。已有最新 tag 等于目标 SHA 时直接报告无变更，不调用脚本。--name 用于同名补推时优先复用相同 SHA 的 tag。消息读取本地分支或跟踪分支且不执行脚本；tag 仍 fetch 并固定远端 SHA。

消息模板的 branch/version 必须绑定用户选择的分支规则，不能再通过跨分支 number() 匹配来确定。version 仅按该分支 tag_prefix 去掉一次前缀，缺省或不匹配时保持完整 tag 名；msg 与 tag 共享同一历史筛选模块。

### Protocol v1

stdin 写入一份 UTF-8 JSON 后关闭；不分多轮、不提供 TTY。输入：

```json
{"version":1,"branch":"5.6","target_sha":"full-commit-id","previous_tag":"release-5.6-9","previous_sha":"full-commit-id","dry_run":false}
```

正常结果：

```json
{"version":1,"action":"tag","name":"release-5.6-10"}
```

或：

```json
{"version":1,"action":"skip","reason":"No release needed"}
```

version 独立于 schema_version、message.version、CLI 版本。响应只接受一个 JSON 对象，允许尾部空白，拒绝未知版本、未知字段、重复字段、错误类型、空名称/原因、多个对象和混入日志。tag 与 skip 字段互斥。skip 是成功项，原因原样展示。全部分支都是 skip（或无变更）时退出码为 0；任一失败时仍退出非零。stderr 用于日志，不参与协议解析；非零退出即失败，即使 stdout 看起来有效。

### Execution and ownership

每个分支顺序调用一次。--dry-run 也调用，并传 dry_run=true；脚本需要只计算，不能把 dry_run 当成 rls 提供的副作用隔离。脚本文件来自当前 worktree，不是 target_sha 中的 blob，不创建临时检出。工作区可能脏或本地领先，实际运行的脚本版本可能与发布提交不同，首版接受此行为。脚本可读取工作区文件，但若名称需要发布文件版本，应读取 target_sha 中的文件，例如 git show，而非假设当前文件代表发布目标。

runner 并发排空 stdout/stderr，避免管道阻塞。stdout 上限 64 KiB；stderr 保留末尾 64 KiB，继续排空并标记截断。超时默认 10 秒，覆盖整个进程与管道收尾；首版仅支持 Unix 脚本执行：脚本进入独立进程组，超时或 stdout 超限时终止该组并回收直接子进程，关闭管道，有界返回。脚本不得主动脱离进程组；本机制不承诺追踪自行 setsid 脱离的任意代码。Windows 在需要调用脚本时明确报不支持，并在 fetch 前结束该分支，不启动脚本；不提供只杀直接子进程的降级执行。Windows 上 msg 和 --name 不运行脚本，仍可使用配置的历史规则。stdin 提前关闭、启动失败均报告分支及原因。继承环境与 PATH，但不将环境值写入诊断；不声称隔离脚本行为。

只有成功的 tag 响应进入 Git 校验：合法 ref、匹配当前分支 tag_pattern（数字模式则匹配数字规则）、不在 exclude_tags、名称未被不同 SHA 占用。创建新 tag 时，存在 previous 则目标 SHA 必须是其严格后代；首次没有 previous 时不要求祖先基线。不满足 pattern 的脚本结果不能发布，避免下次 msg 无法发现。脚本不能改变 SHA、remote 或 push。同名且同 SHA 的已有 tag 可复用补推，此时不要求目标严格晚于自身；不同 SHA 不覆盖，也不允许另起新名称在同一提交创建空版本。创建前重查远端分支。部分失败保持现有逐项结果与非零退出，不回滚成功项。

--name 绕过脚本并执行相同校验，适用于指定名称及推送失败重试。错误需打印可复制的原名称重试命令，不能重新调用脚本生成另一个版本来掩盖失败。无 pattern 的数字模式还须保持数字单调；有 pattern 的祖先链不限制名称字典序。

## Risks / Trade-offs

脚本是任意代码，dry-run 也可能有脚本自身副作用；首次使用应检查配置的实际脚本。无变更短路意味着不能仅靠脚本为相同提交创建第二个发布 tag。Git 历史分叉选择报错而非猜测，必要时使用明确的 exclude_tags。Unix 进程组终止必须验证；Windows 脚本支持延后到进程树回收实现后。

## Migration Plan

先实现独立历史选择与 config，再实现 runner 和 tag 接入；最后同步帮助、英文验收、i18n、README 与 skill。全部验证通过前不把 script 模式描述成现有能力。
