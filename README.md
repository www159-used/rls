# rls

面向多分支、多技术栈项目的发布 CLI：生成发布计划、执行发布动作，再用 changelog 和模板生成可复制的消息。

状态：已实现可配置的 `message` 命令，以及 OpenSpec 与 Cucumber 基础设施。`plan`、`run` 和发布后端尚待实现。

## 为什么单独做

`crsu` 负责代码评审与合入，`rls` 负责版本和发布。发布流程以 Git 提交为输入，不依赖 Crucible、Jira 或某一种语言。

首期提供 `git-tag` 后端，覆盖维护分支打补丁 tag 的场景。随后接入 `cargo release`，由它负责 Rust 包的版本和发布动作。非 Rust 项目可以使用 `git-tag`，有版本文件或制品发布需求时再增加对应后端。

## 预期用法

```bash
# 读取配置，核对远端提交，生成可审阅的发布计划
rls plan 5.6 5.6-2sync

# 执行已保存的计划；是否推送由计划中的动作决定
rls run <plan-id>

# 从已记录的发布结果重新生成消息，并复制到剪贴板
rls message <release-id> --copy
```

计划包含版本、目标提交、前后 tag、changelog 和实际执行动作。发布结果可以再次套用模板，无需重复打 tag。

## 项目资料

- [产品目标、架构与实施顺序](openspec/project.md)
- [开发流程规范](openspec/specs/development-workflow/spec.md)
- [仓库配置示例](rls.example.toml)
- [全局消息配置示例](rls.global.example.toml)
- [消息模板示例](templates/release-message.md.j2)

使用 Rust 实现 CLI。第一版采用内置后端接口，不建设动态插件系统，也不提供默认发送聊天消息的功能。

消息支持自定义模板和变量。默认读取共享 Git 元目录的 `rls/config.toml`（普通仓库为 `.git/rls/config.toml`），继承全局消息设置，仓库同名字段优先。全局目录沿用系统用户配置目录约定，支持 `RLS_CONFIG_HOME` 覆盖；macOS 默认为 `~/Library/Application Support/rls`。相对模板路径以声明它的配置文件目录为基准。详见 [配置规则与验收场景](openspec/project.md)。

## 安装与更新

```bash
cd /Users/ww/Documents/self/rls
make install
export PATH="$HOME/.local/bin:$PATH"
rehash
rls --version
```

需要 Rust/Cargo 和 make。后续更新代码后再次运行 `make install` 即可；默认安装到 `~/.local/bin`，可通过 `make install PREFIX=/自定义目录` 修改。确保 `~/.local/bin` 位于 PATH 前面；系统可能有 Rust 工具链自带的同名 `rls`，可用 `command -v rls` 检查，预期为 `~/.local/bin/rls`。安装后可以直接运行 `rls message --record <记录.json>`。

运行 `make` 查看可用命令；`make build` 构建 release 版本，`make test` 运行测试，`make check` 检查格式、Clippy 与 OpenSpec（需安装 OpenSpec）。

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

可以对 Agent 说：“使用 rls skill，为当前仓库配置消息模板，并从这份发布记录生成通知。”skill 会检查安装版本，明确 `plan/run/nexttag` 尚未实现，不把示例记录当作真实发布。

`make test-skill` 在临时目录验证安装、更新保护、自定义路径和各 Agent 的目录选择；未启动各厂商 CLI 做运行时加载测试。

## 生成消息

```bash
# 用示例记录体验；示例不代表真实发布
cargo run --locked -- message --record examples/release-record.json

# 输出文件；加 --copy 可同时复制到系统剪贴板
cargo run --locked -- message --record examples/release-record.json --output /tmp/release-message.txt

# 当前仓库根目录 .rls/state/releases/demo.json 中的记录
cargo run --locked -- message demo
```

`--record` 支持仓库外运行；按 ID 查找时使用当前 worktree 的状态目录。记录格式见 [JSON 示例](examples/release-record.json)：`schema_version = 1`，包含 `release_id` 和 `releases`。每项需要 `status`（`success`、`failed`、`skipped`）、`branch`、`version`、`tag`、`sha`、`commits`；每条提交包含 `sha` 和 `subject`。仅成功项进入模板，其他项在 stderr 单独报告。

仓库配置与模板可以这样安装（已有文件请直接编辑）：

```bash
rls_metadata="$(git rev-parse --path-format=absolute --git-common-dir)/rls"
mkdir -p "$rls_metadata/templates"
cp -n rls.example.toml "$rls_metadata/config.toml"
cp -n templates/release-message.md.j2 "$rls_metadata/templates/release-message.md.j2"
```

全局配置使用同样的文件布局，参见 [全局配置示例](rls.global.example.toml)。`[message]` 使用独立的 `version = 1` 标记消息配置格式，与顶层 `schema_version` 和发布版本无关。旧配置省略时按 v1 读取；每层先校验版本再合并，未知版本报错，不由另一层覆盖掉错误。消息字段优先级为仓库、全局、内置；`message.variables` 按顶层键合并，空字符串覆盖原值。模板可访问 `releases`、`release_id` 和 `variables`，支持 Jinja 风格循环、条件与格式化。变量拼错、模板丢失、无效配置都会失败并显示文件来源。模板不提供文件加载、shell 或环境变量访问。

每次执行都会重新读取消息设置；修改文案后直接重跑即可。输出文件与剪贴板失败不会修改发布记录，文件输出也不能覆盖输入记录。`--copy` 需要可用的桌面剪贴板。

## 开发与验收

本仓库已用 OpenSpec 1.6.0 初始化 Codex 工作流，使用 cucumber-rs 0.23 执行中文 Gherkin。开发需要 Rust/Cargo（本次验证为 Rust 1.95.0）；OpenSpec 需要 Node.js 20.19+。

```bash
# 安装与本仓库初始化时相同的 OpenSpec CLI
npm install -g @fission-ai/openspec@1.6.0

# 校验规范和变更
openspec validate --all --strict --no-interactive

# 执行验收场景，依赖版本由 Cargo.lock 固定
cargo test --locked --test acceptance

# 仅执行配置示例场景
cargo test --locked --test acceptance -- --tags @configuration-example

cargo fmt --check
```

在 Codex 中使用 `$openspec-propose` 提出变更，再用 `$openspec-apply-change` 实施、`$openspec-archive-change` 归档；新生成的技能可能需要重启会话才能发现。规范放在 `openspec/specs/`，进行中的变更放在 `openspec/changes/`，产品上下文与产物规则在 `openspec/config.yaml`。

新增行为先写 OpenSpec 需求和场景，再添加 `tests/features/*.feature` 及 `tests/acceptance.rs` 中的步骤。步骤失败或未定义都会使测试失败。发布测试应使用临时仓库和本地 bare remote。

验收覆盖配置示例、真实仓库与 linked worktree 的消息配置、模板错误和 CLI 输出。发布流程及计划指纹尚未实现。

参考：[OpenSpec 项目初始化](https://openspec.dev/docs/setup)、[Cucumber Rust 入门](https://cucumber-rs.github.io/cucumber/main/quickstart.html)。

## 许可证

本项目采用 GNU AGPL v3（`AGPL-3.0-only`），英文原文见 [LICENSE](LICENSE)，来源为 [GNU 官方原文](https://www.gnu.org/licenses/agpl-3.0.txt)。
