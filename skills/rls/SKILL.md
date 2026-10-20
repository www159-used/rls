---
name: rls
description: Use the rls release CLI to generate or regenerate release messages from Git tags and commits, configure repository or global message templates, and troubleshoot message output. Use when the user asks to use rls for release notifications or configure its messages; not for Rust Language Server tasks.
license: AGPL-3.0-only
---

# rls 发布消息

帮助用户在目标项目中配置和生成发布消息。此 skill 适用于提供 shell 和文件访问的 Agent CLI，不依赖特定 Agent 的工具名称、用户路径或其他 skill。

## 确认命令与能力

先运行 `rls --help` 和 `rls message --help`，确认找到的是发布 CLI。某些机器上的同名 `rls` 是 Rust Language Server；必要时检查 `command -v rls`（POSIX）或 `Get-Command rls`（PowerShell），使用已安装发布 CLI 的完整路径。

当前实现支持 `message`；`plan`、`run`、nexttag 和 Git 发布后端尚未实现。以实际安装版本的帮助为准。不要把规划中的命令当成可执行功能，也不要为完成消息任务擅自打 tag、push 或发送聊天消息。

若工具未安装且有 rls 源码检出，在源码目录执行 `make install`（需要 Rust/Cargo 和 make），默认安装到 `~/.local/bin`；无 make 时使用 `cargo install --path . --locked --root <安装根目录>`。目标项目仓库不一定是 rls 源码仓库，不要在错误目录安装；缺少源码或安装位置时询问缺失信息。

## 生成或修改消息

在用户指定的目标仓库执行命令，让仓库配置参与解析。

```bash
# 显式区间，包含基线之后直到目标的全部提交
rls message <目标tag> --from <基线tag>

# 根据仓库版本线自动选择上一 tag
rls message <目标tag>

# 多个目标分别选基线，按参数顺序输出
rls message <目标tag1> <目标tag2>

# 缩写、文件和剪贴板
rls msg <目标tag> -o <message.txt>
rls m <目标tag> -c
```

位置参数是已有 tag，不是版本号、分支或记录 ID。`--from` 只适用于单个目标。没有 records、JSON 输入或 `--record` 参数，不要要求用户准备这些文件。

自动选择需要仓库 branches 规则，详见 [版本线和模板格式](references/formats.md)。只选择目标祖先中的同版本线 tag，按数字而非字符串排序；没有基线、历史不单调或歧义时显式指定 --from。浅克隆需要补全历史或显式选择可验证的基线。基线必须是目标祖先，同一提交报告无新变更。

命令只读取本地 Git，不自动 fetch。如果 tag 不在本地，说明缺失的 tag，并按用户任务范围获取需要的引用。轻量和带注释 tag 均支持。本地 tag 存在不能证明远端发布成功，不能据此宣布发布完成。

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

改完后用用户选定的 tag 区间实际渲染，检查称呼、版本、完整提交顺序和空行。交付生成文本或输出文件路径；消息生成本身不代表通知已经发出。
