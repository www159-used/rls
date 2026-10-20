---
name: rls
description: Use the rls release CLI to create incremental tags across branches, generate or regenerate release messages from Git tags and commits, configure repository or global message templates, and troubleshoot message output. Use when the user asks to use rls for release notifications or configure its messages; not for Rust Language Server tasks.
license: AGPL-3.0-only
---

# rls 发布消息

帮助用户在目标项目中配置和生成发布消息。此 skill 适用于提供 shell 和文件访问的 Agent CLI，不依赖特定 Agent 的工具名称、用户路径或其他 skill。

## 确认命令与能力

先运行 `rls --help` 和 `rls message --help`，确认找到的是发布 CLI。某些机器上的同名 `rls` 是 Rust Language Server；必要时检查 `command -v rls`（POSIX）或 `Get-Command rls`（PowerShell），使用已安装发布 CLI 的完整路径。

当前实现支持 `message/msg/m` 与 `tag/t`；`plan`、`run` 和 Cargo 发布后端尚未实现。以实际安装版本的帮助为准。不要把规划中的命令当成可执行功能，也不要为完成消息任务擅自打 tag、push 或发送聊天消息。

若工具未安装且有 rls 源码检出，在源码目录执行 `make install`（需要 Rust/Cargo 和 make），默认安装到 `~/.local/bin`；无 make 时使用 `cargo install --path . --locked --root <安装根目录>`。目标项目仓库不一定是 rls 源码仓库，不要在错误目录安装；缺少源码或安装位置时询问缺失信息。

## 生成或修改消息

在用户指定的目标仓库执行命令，让仓库配置参与解析。

```bash
rls msg 5.6 5.6-2sync       # 多分支最近两个 tag 的消息合并
rls m                     # 默认当前分支
rls m 5.6 -o message.txt
rls m 5.6 -c
```

位置参数为配置中的分支，不再支持 tag、range 或 --from。每个分支使用其可达历史中同版本线数字最大的两个 tag，读取上一 tag 到最新 tag 的提交，排除尚未打 tag 的提交。按照输入顺序一次渲染；任一分支失败不输出半条消息。至少两个 tag，浅克隆、历史不单调、同数字歧义或空区间必须处理后重试。

消息读取本地分支，缺失时读取配置远端的跟踪分支；不自动 fetch。需要新引用时按用户任务范围获取。本地 tag 不等于推送成功。分支与模板规则见 [格式参考](references/formats.md)。

## 创建 tag

只有用户要求创建或推送 tag 时使用此流程；消息任务不需要打 tag。

```bash
rls t 5.6 5.6-2sync -n       # 会 fetch，但不创建发布 tag 或推送
rls t 5.6 5.6-2sync          # 创建本地 tag
rls t 5.6 5.6-2sync -p       # 创建并推送所选 tags
rls t 5.6 --name v5.6.0.10 -p # 单分支按指定名称创建或重试
```

按远端分支 SHA 操作，不打到本地领先提交；默认 origin，可用 -r/--remote 覆盖。counter 默认末位加一；manual 或首次没有基线用 --name，名称必须匹配版本线。默认不推送，只有 -p/--push 启用，不能用 git push --tags 代替。

检查每个分支的结果及总退出码。批量不是事务，部分失败保留成功项，不自动删除 tag；推送失败保留本地 tag，用相同 --name 重试，不能重新递增来掩盖失败。冲突不同 SHA 不覆盖、不强推；远端分支变化后重新评估，不能移动已发布 tag。

## 语言和补全

界面语言支持 `--lang en|zh-CN` 或 `RLS_LANG`，默认跟随系统 locale。CLI 语言不改变用户模板或提交内容。安装源码中的手写补全使用 `make install-completions`，安装到 PREFIX/share 下；Zsh 将 `PREFIX/share/zsh/site-functions` 加入 fpath 后 compinit，Bash source `PREFIX/share/bash-completion/completions/rls`，Fish 使用其 completions 目录。补全通过只读 `completion-branches` 获取配置分支，不执行发布。

消息由 Git 的 SHA、tag 和完整提交区间生成。修改称呼、模板或空行后直接重跑相同命令；配置变量只进入 `variables`，不改变 Git 数据。检查退出码和 stderr，模板、文件或剪贴板失败不能报告为成功；可独立重试输出，无需发布操作。

## 定位并编辑配置

仓库配置为 `<git-common-dir>/rls/config.toml`。使用 Git 发现共享元目录：

```bash
git rev-parse --path-format=absolute --git-common-dir
```

普通仓库对应 `.git/rls/config.toml`；linked worktree 共用配置。不要直接假设 `.git` 是目录。工作区根目录的 `rls.toml` 不参与默认发现。

全局配置为 `$RLS_CONFIG_HOME/config.toml`（变量非空时）；否则使用系统用户配置目录下的 `rls/config.toml`：macOS `~/Library/Application Support/rls`、Linux `${XDG_CONFIG_HOME:-~/.config}/rls`、Windows `%APPDATA%\rls`。该目录与 Agent 的 skill 安装目录无关。

用户指定仓库或全局时遵循其范围；只给当前项目改文案时修改仓库层，保留其他设置。不要用整份示例覆盖现有配置。最小消息配置示例和模板见 [格式参考](references/formats.md)。

- `message.version` 标记消息配置格式，当前为整数 `1`；省略按 v1。它不同于顶层 `schema_version`、CLI 版本和发布版本。
- 各层先独立校验版本，再合并。未知版本或损坏文件报错，不因另一层覆盖而忽略。
- 模板优先级：仓库 → 全局 → 内置。`message.variables` 按顶层键覆盖，显式空字符串可以清空文案。
- 相对模板路径以**声明该模板的配置文件目录**为基准；继承全局模板时仍使用全局目录。
- 变量未定义、模板缺失或语法错误均失败。模板没有 shell、环境变量访问或任意文件加载功能。

改完后用用户选定的分支实际渲染，检查称呼、版本、完整提交顺序和空行。交付生成文本或输出文件路径；消息生成本身不代表通知已经发出。
