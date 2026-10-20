# rls

多分支发布 CLI：按远端提交递增 Git tag，从各分支最近两个 tag 之间的提交生成可配置消息。提供 Bash、Zsh、Fish 补全和中英文界面。

```bash
rls t 5.6 5.6-2sync -n       # 预览，会 fetch 引用
rls t 5.6 5.6-2sync          # 仅创建本地 tag
rls t 5.6 5.6-2sync -p       # 创建并推送所选 tag
rls m 5.6 5.6-2sync -c       # 合并消息并复制
```

`tag/t` 和 `message/msg/m` 均接受分支列表，省略使用当前分支；detached HEAD 需显式指定。重复分支报错。当前不支持 `plan/run`、制品发布或 Cargo 后端；本项目自身的 cargo-release 工具配置是独立的开发流程。

## 安装与更新

```bash
cd /path/to/rls
make install
export PATH="$HOME/.local/bin:$PATH"
rehash
rls --version
```

需要 rustup 和 make。仓库的 `rust-toolchain.toml` 固定 Rust 1.95.0（含 rustfmt、Clippy），Cargo.toml 声明 rust-version = 1.95。后续更新代码后再次运行 `make install` 即可；默认安装到 `~/.local/bin`，可通过 `make install PREFIX=/自定义目录` 修改。确保 `~/.local/bin` 位于 PATH 前面；系统可能有 Rust 工具链自带的同名 `rls`，可用 `command -v rls` 检查，预期为 `~/.local/bin/rls`。安装后可以直接运行 `rls msg <分支>`。

运行 `make` 查看可用命令；`make build` 构建 release 版本，`make test` 运行测试，`make check` 检查格式、Clippy 与 OpenSpec（需安装 OpenSpec）。

## 维护本项目的版本

本项目使用 cargo-release **1.1.2** 管理 Cargo.toml / Cargo.lock 的版本号和版本提交。这是 rls 自身的开发流程，目标项目的 Cargo 发布后端仍未实现。

```bash
# 一次性安装开发工具；本机已有该版本时无需重复执行
make setup-release

# 先提交当前代码改动，保持工作区干净，再验证
make check test

# 预览：默认 patch，也支持 minor、major 或显式版本号
make release
make release LEVEL=0.2.0

# 实际更新版本并创建本地版本提交，保留 cargo-release 的确认提示
make release-execute LEVEL=patch
```

`release.toml` 显式启用版本管理，同时关闭 `tag`、`push`、`publish`。Makefile 使用 `--isolated --config release.toml`，不继承用户全局 cargo-release 设置。`make release` 仅 dry-run；`make build` 仍是编译优化二进制，二者不同。

**tag 由你手动创建**，确认版本提交后再操作（以下版本号按实际修改）：

```bash
git tag v0.1.1
# 确认目标远端与分支后再自行推送提交和 tag
# 已有前一 tag 时可生成消息
rls msg master # 需先配置 master 的版本线，且有两个 tag
```

首次尚无基线 tag 时不能使用上例生成区间消息。该项目保留 `publish = false`，不会发布到 crates.io。实际发布要求工作区干净；尚未提交的配置改动可能使 cargo-release 检查失败。

参考：[cargo-release 1.1.2 配置](https://github.com/crate-ci/cargo-release/blob/v1.1.2/docs/reference.md)、[rustup 工具链文件](https://rust-lang.github.io/rustup/overrides.html#the-toolchain-file)。

## 给 Agent CLI 安装 skill

项目提供可独立复制的 [rls skill](skills/rls/SKILL.md)，包含消息生成、配置发现、格式版本和错误处理说明。安装需要 Python 3；不会安装 CLI 本身，也不会修改 Agent 的权限或启动配置。

```bash
# 默认安装到 ~/.agents/skills/rls，供支持共享目录的 Agent 使用
make install-skill

# 按使用的 Agent 任选一个；不需要全部安装
make install-skill AGENT=codex
make install-skill AGENT=claude
make install-skill AGENT=gemini
make install-skill AGENT=opencode

# 安装到另一个项目，而非用户全局目录
make install-skill AGENT=claude SCOPE=project PROJECT=/path/to/project

# 其他 Agent 或自定义目录：指定 skills 父目录，安装器自动添加 /rls
make install-skill SKILL_DIR=/path/to/agent/skills

# 更新已有安装（会替换整个 rls skill 目录，包括本地修改）
make install-skill AGENT=claude FORCE=1
```

默认拒绝覆盖已有目录；更新时显式传 `FORCE=1`。安装使用文件复制，移动或删除源码仓库后仍可读取 skill；未来更新后需重新安装。也可手动复制整个 `skills/rls/` 文件夹到目标 Agent 的 skills 目录。没有 shell/make 的环境可用 `python3 scripts/install_skill.py --help` 查看参数，Windows 通常使用 `python`。

| Agent | 用户级目录 | 项目级目录 |
|---|---|---|
| 通用 / Codex | `~/.agents/skills/rls` | `.agents/skills/rls` |
| Claude Code | `~/.claude/skills/rls` | `.claude/skills/rls` |
| Gemini CLI | `~/.gemini/skills/rls` | `.gemini/skills/rls` |
| OpenCode | `${XDG_CONFIG_HOME:-~/.config}/opencode/skills/rls` | `.opencode/skills/rls` |

安装目录依据 [Codex](https://developers.openai.com/codex/skills/)、[Claude Code](https://code.claude.com/docs/en/skills)、[Gemini CLI](https://geminicli.com/docs/cli/skills/) 和 [OpenCode](https://opencode.ai/docs/skills/) 的官方说明。支持情况取决于实际 CLI 版本；安装后重启会话或使用其 skills 重载功能。旧版或不同目录布局使用 `SKILL_DIR` 覆盖。

可以对 Agent 说：“使用 rls skill，为当前仓库配置消息模板，并为指定分支生成通知。”skill 会检查安装版本，明确 `plan/run` 尚未实现，不把本地 tag 的存在当作已推送或发布成功。

`make test-skill` 在临时目录验证安装、更新保护、自定义路径和各 Agent 的目录选择；未启动各厂商 CLI 做运行时加载测试。

## 配置和消息

仓库配置为 `<git-common-dir>/rls/config.toml`，普通仓库对应 `.git/rls/config.toml`；linked worktree 共用。使用 `git rev-parse --path-format=absolute --git-common-dir` 发现路径。

```toml
schema_version = 1
[release]
remote = "origin"
[branches."5.6"]
version_prefix = "5.6.0"
tag_prefix = "v"
increment = "counter"
[branches."5.6-2sync"]
version_prefix = "5.6.1"
tag_prefix = "v"
increment = "counter"
[message]
version = 1
# template = "templates/release-message.md.j2"
[message.variables]
greeting = "请协助构建以下补丁包，谢谢。"
```

`message.version` 与顶层 `schema_version` 独立，省略按 v1，各层合并前分别校验。全局配置来自 `$RLS_CONFIG_HOME/config.toml`，否则位于系统用户配置目录下的 `rls/config.toml`；macOS 为 `~/Library/Application Support/rls`。消息模板按仓库、全局、内置顺序选择，变量按键覆盖；相对模板路径以声明目录为基准。分支及远端规则仅读取仓库配置。

`rls msg 5.6 5.6-2sync` 按参数顺序将各分支的上一 tag 到最新 tag 区间一次渲染为一条消息，不包含最新 tag 之后的提交。不再接受 tag 参数、range 或 `--from`。`-o/--output` 写文件，`-c/--copy` 复制。命令只读本地 Git；优先使用本地分支，没有时使用配置远端的跟踪分支，不自动 fetch。

按版本线严格匹配 `tag_prefix + version_prefix + .整数`，只考虑分支可达的 tag，按数字排序取最近两个。至少需要两个 tag；浅克隆、重复数字、历史倒退、分叉或同一提交的空区间报错。任一分支失败时不输出部分消息。模板接收 `releases` 和 `variables`，每项 release 含 branch、version、tag、previous_tag、sha、previous_sha、commits（sha、subject），提交按反向拓扑排序。不使用 records，也不自动发送消息。

完整示例：[仓库配置](rls.example.toml)、[全局配置](rls.global.example.toml)、[模板](templates/release-message.md.j2)。复制模板时保留相对目录；配置变量不会改变 Git 事实。

## tag 递增和重试

`tag` 获取所选远端分支与 tags，以远端提交为目标，不切换分支或改动工作区。默认 counter 模式取可达的同线最大数字加一（9 → 10）。无新提交不创建空版本；名称冲突不覆盖、不跳号、不强推。`increment = "manual"` 要求显式 `--name`，名称仍需符合版本线格式；首次没有基线也使用 `--name`。

```bash
rls t 5.6 -n                      # 只预览发布动作；fetch 会更新本地引用
rls t 5.6 --name v5.6.0.1         # 首次发布或手工指定版本
rls t 5.6 --name v5.6.0.10 -p     # 按原名称补推，存在且同 SHA 可复用
rls t 5.6 -r upstream -p          # 覆盖配置远端
```

默认仅创建轻量 tag，只有 `--push/-p` 才推送，配置中的旧 `push` 字段不启用推送。只推送本次所选 tag，不推送分支或其他 tags。`--name` 限单分支。多分支逐项执行，部分失败返回非零；保留已创建/已推送结果，不自动回滚。推送失败后以输出的 tag 名和 `--name` 重试；重试仍验证该 tag 指向当前远端分支提交，分支变化时报错。

## 补全与语言

项目手写维护 `completions/` 中的 Bash、Zsh、Fish 脚本，包含别名、选项、仓库配置分支、远端名、已有 tag 和输出路径补全。补全只读，不 fetch 或发布。

```bash
make install-completions
# Bash：写入 ~/.bashrc（自定义 PREFIX 时修改路径）
source "$HOME/.local/share/bash-completion/completions/rls"
# Zsh：写入 ~/.zshrc，fpath 必须在 compinit 前设置
fpath=("$HOME/.local/share/zsh/site-functions" $fpath)
autoload -Uz compinit && compinit
# Fish：可将脚本复制到用户自动加载目录
mkdir -p ~/.config/fish/completions
cp completions/rls.fish ~/.config/fish/completions/rls.fish
```

语言优先级：`--lang` → `RLS_LANG` → `LC_ALL` → `LC_MESSAGES` → `LANG`。系统中文环境使用中文，其余默认英文；显式支持 `en`、`zh-CN`（别名 `zh`）。界面语言影响帮助、执行状态及项目错误提示，不翻译模板正文、提交、Git 原始诊断或依赖库错误。

```bash
rls --lang en tag --help
rls msg 5.6 --lang zh-CN
export RLS_LANG=zh-CN
```

## 开发与规范

`make test` 执行英文 Cucumber、CLI 语言测试及 Python 安装/补全测试；`make check` 执行 fmt、Clippy 和 OpenSpec 严格校验。Git 验收使用临时真实仓库和 bare remote。OpenSpec CLI 使用 `@fission-ai/openspec@1.6.0`。

参见 [产品纲要](openspec/project.md) 与 [规范](openspec/specs/)。新增命令同时考虑别名、补全、i18n、skill 和真实 Git 行为验收。

## 许可证

GNU AGPL v3（`AGPL-3.0-only`），英文原文见 [LICENSE](LICENSE)。
