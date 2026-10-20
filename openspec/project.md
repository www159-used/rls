# 产品目标与设计上下文

rls 是独立的多分支发布 CLI。当前已实现基于 Git tag 的消息生成、分层配置与通用 Agent skill。已实现多分支 tag；`plan`、`run` 和 Cargo 后端仍待实现。

## 已实现：多分支消息和 tag

- `message/msg/m [branch]...` 接受分支列表，省略为当前分支；detached HEAD 要求显式分支。旧 tag 参数、range、--from 不再支持。
- 每个分支选择可达的同版本线数字最大的两个 tag，读取上一 tag 到最新 tag 的全部提交，排除最新 tag 后的提交；按分支参数顺序一次渲染。优先本地分支，缺失时使用配置远端跟踪分支。只读、不 fetch，无 records。
- 至少两个 tag；浅克隆、数字重复、非单调或分叉历史、空区间报错。任一分支失败不输出部分消息。输出支持 -o/--output 与 -c/--copy。
- `tag/t [branch]...` fetch 所选远端分支和 tags，以远端 SHA 创建轻量 tag，不切换工作区。`release.remote` 默认 origin，-r/--remote 覆盖。
- `branches.<name>.increment` 支持 counter（默认，版本末位数字加一）和 manual（要求 --name）。首次无基线使用 --name；名称必须符合版本线，--name 仅单分支。无变更不发空版本，冲突不覆盖、不跳号、不强推。
- -n/--dry-run 会 fetch 引用，但不创建发布 tag 或推送。仅 -p/--push 启用推送，只推选定 tag。批量逐项执行，部分失败退出非零并保留成功项；同 SHA 同名 tag 可用 --name 补推，远端分支变化后重新评估。

## CLI 一致性

新增命令必须同时设计全称、无歧义的显式别名、帮助、补全及 skill；不支持自动前缀缩写。项目手写维护 Bash/Zsh/Fish 补全，补全操作只读，通过隐藏 completion-branches 获取配置分支。make install-completions 安装脚本。

界面语言优先级为 --lang、RLS_LANG、LC_ALL、LC_MESSAGES、LANG，中文 locale 使用中文，其余英文。显式支持 en、zh-CN/zh，未知显式值报错。语言不改变消息模板、Git 事实和依赖原始诊断。测试场景、步骤和常规数据使用英文，中文仅保留在 i18n 验证中。

## 配置与模板

仓库配置位于共享 Git 元目录的 `rls/config.toml`。普通仓库对应 `.git/rls/config.toml`；linked worktree 共用配置，工作区根目录的 `rls.toml` 不参与发现。

全局配置为非空 `RLS_CONFIG_HOME` 指定目录中的 config.toml，否则采用系统用户配置目录下的 rls/config.toml：Linux `${XDG_CONFIG_HOME:-~/.config}/rls`，macOS `~/Library/Application Support/rls`，Windows `%APPDATA%\rls`。

顶层 schema_version 当前为 1；message.version 独立描述消息格式，当前为整数 1，省略按 v1。各层在合并前独立校验版本，不因其他层覆盖而忽略错误。

- 模板选择：仓库优先于全局，再回退内置模板。
- message.variables 按顶层键合并，仓库同名键覆盖全局；显式空字符串可清空称呼。
- 相对模板路径以声明该字段的配置目录为基准；继承全局模板保留全局基准。
- branches 规则只读取仓库层，避免全局版本线误用于其他项目。
- 配置缺失时允许回退；损坏文件、未知版本、显式模板缺失或变量未定义必须报错并指出来源。

MiniJinja 模板提供 `releases` 和 `variables`。每项 release 包含 tag、previous_tag、sha、previous_sha、version、branch、commits；每条 commit 包含 sha 和 subject。branch 是请求的配置键；version 按规则去掉 tag_prefix。

不存在 release_id 或 status 字段。消息配置不能覆盖 Git 事实数据，模板不执行 shell、不读取环境变量或加载任意文件。内置模板只包含 tag 和 changelog，称呼等通过自定义模板引入。修改元数据后直接重跑消息命令即可。

## 模块边界

- `config`：发现仓库/全局设置，校验版本，合并消息字段，保留路径来源。
- `git`：校验 tag、解析 SHA、寻找基线、验证祖先关系、读取提交。
- `message`：将内存中的 Git 区间数据与变量渲染为文本。
- CLI：解释参数，调用模块，输出终端、文件和剪贴板。

版本与标签操作以结构化参数调用 Git，不把配置插值为 shell。首期维持单一 Rust crate，不建设动态插件系统，不内置个人信息或自动发送聊天消息。

## 本项目开发工具

`rust-toolchain.toml` 固定 Rust 1.95.0，使用 minimal profile 加 rustfmt、Clippy；Cargo 声明 rust-version = 1.95。

本项目使用 cargo-release 1.1.2 管理版本更新和本地版本提交，release.toml 显式 release/tag/push = true、publish = false，push-remote = origin。make release 仅预览，make release-execute 更新版本、创建 tag 并推送 origin，保留确认提示。此工具配置不代表 rls 的目标项目 Cargo 发布后端已经实现。

## 后续：发布规划与执行

发布计划仍以指定远端分支为准，不把本地领先提交当作已合入。`plan` 可以获取远端引用，但不创建 tag、不修改源码或发布制品。执行前需核对远端 SHA、基线和发布相关配置；消息文案变化不应让计划过期。

Cargo 后端在第二阶段验证：可能修改版本文件、产生发布提交，需隔离检出并明确 tag 动作归属；复用 cargo-release，不复制其版本更新与发布逻辑。用真实 workspace 验证多包结果后再定型后端 trait。

## 实施顺序

- [x] OpenSpec、Cucumber、分层消息配置、可配置模板与输出。
- [x] 从 Git tag 自动读取消息内容，支持显式区间与按版本线选基线。
- [x] 通用 Agent skill、跨 Agent 安装器与 AGPL v3 许可证。
- [x] 多分支 tag 递增、显式推送、冲突保护和重试，使用本地 bare remote 验收。
- [x] 多分支消息、手写 shell 补全、中英文界面和英文测试。
- [ ] 按实际需求增加持久化 plan/run 与过期计划检查。
- [ ] cargo-release 单包及 workspace 集成。

当前验收在临时真实 Git 仓库中运行，覆盖注释 tag、数字排序、多个版本线、祖先关系、无变更、worktree、未提交文件保持、配置继承及模板错误。tag 推送已用本地 bare remote 验收；planner 指纹尚未实现。

## 脚本自定义 tag

已实现 script 模式，使用 version=1 的 JSON stdin/stdout 交互；脚本决定名称或 skip，rls 负责校验、创建和显式推送。--name 绕过脚本恢复。script 必须有 tag_pattern，按祖先链选历史；manual 可选 pattern，counter 保留数字规则。msg 不执行脚本，version 按所选分支 tag_prefix 去前缀。脚本从当前 worktree 根执行，Unix 使用有界输出、超时及独立进程组回收；Windows 在执行前拒绝脚本。详细协议见 README 与 script-tag-increment 规范。
