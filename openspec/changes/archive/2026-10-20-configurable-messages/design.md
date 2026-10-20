## Context

沿用 crsu 的 Git common dir 布局，采用消息字段合并。发布后端尚未实现，消息能力以明确的 JSON 记录作为输入边界。

## Goals / Non-Goals

实现配置发现、模板选择、变量合并、严格渲染、重新生成及独立输出。不执行 tag/push、不实现发布配置校验或 planner 指纹。

## Decisions

- `config` 只消费 schema_version 与 message；消息内部未知字段报错，其他发布设置留给未来 planner。
- 使用 `git rev-parse --path-format=absolute --git-common-dir` 发现共享目录；只有明确的非仓库错误才回退全局。
- 使用 BaseDirs 的系统配置目录加 rls，Windows 对齐 `%APPDATA%/rls`；非空 RLS_CONFIG_HOME 直接覆盖。
- 变量按顶层键合并，保留 TOML 类型和显式空值；相对模板路径在加载声明层时绑定。
- MiniJinja 严格未定义变量，不注册文件 loader、环境访问或 shell 函数，保留尾部换行。
- JSON 记录包含 schema_version、release_id、releases；每项显式 status，只有 success 进入模板。failed/skipped 单独输出 stderr。
- ID 定位当前 worktree 根目录 `.rls/state/releases/<id>.json`，配置跨 worktree 共享，状态保持工作区本地。显式 `--record` 可在仓库外使用。
- 输出失败不修改记录；文件输出拒绝覆盖记录本身（包括硬链接和符号链接）。

## Risks / Trade-offs

记录格式是消息消费边界，未来 runner 可扩充字段；不将示例记录宣称为真实发布。剪贴板依赖桌面会话，失败可独立重试。配置指纹属于未来 planner，本次不伪造该能力。

## Migration Plan

原配置示例仍兼容，复制到共享 Git 元目录并同时复制模板。旧的根目录 rls.toml 不自动发现。
