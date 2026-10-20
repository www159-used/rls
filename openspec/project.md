# 产品目标与设计上下文

rls 是独立的多分支发布 CLI。当前已实现基于 Git tag 的消息生成、分层配置与通用 Agent skill。`plan`、`run`、nexttag 和发布后端仍待实现。

## 已实现：消息生成

用户手工编辑 `<git-common-dir>/rls/config.toml`，只维护模板、文案和版本线规则。版本、tag、SHA 和 changelog 从 Git 自动读取，不需要发布 records、JSON 或记录 ID。

- `rls message <tag>...` 按参数顺序生成各目标 tag 的消息；别名 `msg`、`m`。
- `--from <tag>` 显式指定单个目标的基线。输入必须为已有 tag，轻量和带注释 tag 均支持。
- 自动基线匹配仓库 branches 配置的 `tag_prefix + version_prefix + .<整数>`；在目标祖先中按末位数字比较，排除其他版本线、日期 tag 和不相关历史。
- 目标 tag 必须只匹配一个版本线。首次无基线、历史不单调、数字重复或历史分叉造成歧义时要求显式基线。浅克隆不自动选择基线。
- 显式基线必须是目标提交的祖先；两个 tag 指向同一提交时报告无新变更。
- changelog 使用 `previous_sha..target_sha` 全部提交（包含合并提交），按反向拓扑顺序输出 subject 并保留 SHA，不要求 Conventional Commits。
- 命令只读本地 Git，不 fetch、不切换分支、不 stash、不改工作区、不创建或推送 tag。本地 tag 不代表远端发布已成功。
- 文本输出终端或 `--output/-o` 文件，`--copy/-c` 可复制到剪贴板；失败可独立重试。

## 配置与模板

仓库配置位于共享 Git 元目录的 `rls/config.toml`。普通仓库对应 `.git/rls/config.toml`；linked worktree 共用配置，工作区根目录的 `rls.toml` 不参与发现。

全局配置为非空 `RLS_CONFIG_HOME` 指定目录中的 config.toml，否则采用系统用户配置目录下的 rls/config.toml：Linux `${XDG_CONFIG_HOME:-~/.config}/rls`，macOS `~/Library/Application Support/rls`，Windows `%APPDATA%\rls`。

顶层 schema_version 当前为 1；message.version 独立描述消息格式，当前为整数 1，省略按 v1。各层在合并前独立校验版本，不因其他层覆盖而忽略错误。

- 模板选择：仓库优先于全局，再回退内置模板。
- message.variables 按顶层键合并，仓库同名键覆盖全局；显式空字符串可清空称呼。
- 相对模板路径以声明该字段的配置目录为基准；继承全局模板保留全局基准。
- branches 规则只读取仓库层，避免全局版本线误用于其他项目。
- 配置缺失时允许回退；损坏文件、未知版本、显式模板缺失或变量未定义必须报错并指出来源。

MiniJinja 模板提供 `releases` 和 `variables`。每项 release 包含 tag、previous_tag、sha、previous_sha、version、branch、commits；每条 commit 包含 sha 和 subject。branch 是匹配规则的配置键，未匹配规则的显式区间为空；version 按匹配规则去掉 tag_prefix，无规则时保留完整 tag 名。

不存在 release_id 或 status 字段。消息配置不能覆盖 Git 事实数据，模板不执行 shell、不读取环境变量或加载任意文件。内置模板只包含 tag 和 changelog，称呼等通过自定义模板引入。修改元数据后直接重跑消息命令即可。

## 模块边界

- `config`：发现仓库/全局设置，校验版本，合并消息字段，保留路径来源。
- `git`：校验 tag、解析 SHA、寻找基线、验证祖先关系、读取提交。
- `message`：将内存中的 Git 区间数据与变量渲染为文本。
- CLI：解释参数，调用模块，输出终端、文件和剪贴板。

版本与标签操作以结构化参数调用 Git，不把配置插值为 shell。首期维持单一 Rust crate，不建设动态插件系统，不内置个人信息或自动发送聊天消息。

## 后续：发布规划与执行

发布计划仍以指定远端分支为准，不把本地领先提交当作已合入。`plan` 可以获取远端引用，但不创建 tag、不修改源码或发布制品。执行前需核对远端 SHA、基线和发布相关配置；消息文案变化不应让计划过期。

nexttag 的约定（尚未实现）：从远端目标 SHA 的祖先中选取同版本线最大的正式 tag，末位数字加一；无新变更不发空版本；候选名称已被其他提交占用时报错，不自动跳号、覆盖或强推。首次发布、非单调历史或歧义要求显式指定基线和版本。

多分支发布不视为事务；部分成功时保留已发布结果，不自动删除远端 tag。重试应探测实际 Git 状态。未来计划/执行状态存储按需求设计，不再要求用户维护消息 records。

Cargo 后端在第二阶段验证：可能修改版本文件、产生发布提交，需隔离检出并明确 tag 动作归属；复用 cargo-release，不复制其版本更新与发布逻辑。用真实 workspace 验证多包结果后再定型后端 trait。

## 实施顺序

- [x] OpenSpec、Cucumber、分层消息配置、可配置模板与输出。
- [x] 从 Git tag 自动读取消息内容，支持显式区间与按版本线选基线。
- [x] 通用 Agent skill、跨 Agent 安装器与 AGPL v3 许可证。
- [ ] 单分支 plan/nexttag，远端 SHA、必需提交和基线检查。
- [ ] 单项 run：tag 创建、推送、冲突处理和幂等恢复，使用本地 bare remote 验收。
- [ ] 多分支发布、部分失败恢复和过期计划检查。
- [ ] cargo-release 单包及 workspace 集成。

当前验收在临时真实 Git 仓库中运行，覆盖注释 tag、数字排序、多个版本线、祖先关系、无变更、worktree、未提交文件保持、配置继承及模板错误。远端发布和 planner 指纹尚未验收。
