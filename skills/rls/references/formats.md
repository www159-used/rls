# 版本线与模板格式

## 配置 v1

仓库和全局配置都需要顶层 `schema_version = 1`。已有发布设置保留，只编辑消息相关字段。

```toml
schema_version = 1

[message]
version = 1
template = "templates/release-message.md.j2"

[message.variables]
greeting = "请协助构建以下补丁包，谢谢。"
```

`template` 可省略以继承全局或内置模板；显式空路径非法。内置模板仅输出成功项的 tag 与 changelog，不使用 greeting；要显示自定义称呼，需要选用包含该变量的模板。

变量支持 TOML 值，按顶层键覆盖而非深度合并。`greeting = ""` 覆盖原称呼为空字符串。模板中的 `variables` 和事实数据 `releases` 是独立命名空间。

模板示例，保存到声明配置目录的 `templates/release-message.md.j2`：

```jinja
{{ variables.greeting }}
{% for release in releases %}{{ release.tag }}
changelog:
{% for commit in release.commits %}{{ commit.subject }}
{% endfor %}{% if not loop.last %}

{% endif %}{% endfor %}
```

模板使用 MiniJinja。可访问 `variables`、`releases`；循环和条件可用，未定义变量严格报错，保留模板尾部换行。模板可以读取每条发布的 branch、version、tag、previous_tag、sha、previous_sha、commits。

## 仓库版本线规则

```toml
[branches."5.6"]
version_prefix = "5.6.0"
tag_prefix = "v"
increment = "counter"
exclude_tags = [] # 精确 tag 名；排除旧版本线的特殊编号，不删除 Git tag。

[branches."5.6-2sync"]
version_prefix = "5.6.1"
tag_prefix = "v"
increment = "counter"
```

这些规则只读取仓库配置，不从全局继承。第一条匹配 `v5.6.0.<非负整数>`。msg 选择分支可达的同线 tag，按数字排序取最近两个；历史倒退、数字重复或分叉歧义时报错，不接受 --from。version 去除 tag_prefix，branch 为请求的配置键。

`exclude_tags` 默认为空，使用完整 tag 名精确匹配，同时作用于 tag 递增和消息区间选择；`--name` 也不能使用被排除的名字。它不会删除或移动 tag，也不忽略 fetch 冲突。仅在确认某个历史 tag 不属于本次正式版本线时配置，不能用来掩盖未调查的历史歧义。

`increment` 支持 counter（默认，末位加一）、manual（必须指定 --name）或 script（自定义命名）。仓库 `[release] remote = "origin"` 指定远端，命令 -r 可覆盖；推送仅由 -p/--push 启用。顶层 schema_version 仍为 1，新字段省略时保持默认。

## 模板数据

每项 release 提供：

| 字段 | 含义 |
|---|---|
| tag / previous_tag | 目标及基线 tag 名 |
| sha / previous_sha | 解析后的提交 SHA |
| version | 去掉所选分支 tag_prefix 的版本，省略或不匹配时保留原名 |
| branch | 匹配的仓库版本线配置键 |
| commits | 基线之后直到目标的全部提交，每条包含 sha、subject |

`releases` 按命令参数顺序排列，commits 使用反向拓扑顺序，包含合并提交。不提供 release_id 或 status，也不保存 records。消息使用的 tag 必须在当前仓库内；linked worktree 共用 Git 引用和配置。

## 脚本命名协议 v1

```toml
[branches."5.6"]
increment = "script"
tag_pattern = "release-5.6-*"
tag_prefix = "release-"
[branches."5.6".script]
command = ["sh", "./scripts/next-tag.sh"]
timeout_seconds = 10
```

script 必须有 pattern，counter 禁止 pattern；manual 可选。存在 pattern 时按祖先链取最近 tag，不能配置 version_prefix，tag_prefix 仅剥离模板 version。pattern 全名匹配，* 和 ? 可跨 /，不支持字符组。exclude_tags 优先排除历史和返回名；不接受 tag_order。msg 不执行脚本。

参数数组不经 shell 拼接，cwd 与相对程序路径以当前 worktree 根为基准，裸程序名通过 PATH 查找。只读取仓库层脚本配置。

stdin：`{"version":1,"branch":"5.6","target_sha":"full-id","previous_tag":null,"previous_sha":null,"dry_run":false}`。有基线时 previous 两字段为字符串。stdout：`{"version":1,"action":"tag","name":"release-5.6-new"}` 或 `{"version":1,"action":"skip","reason":"Not scheduled"}`。协议版本与其他配置版本独立。不能输出额外字段、重复字段或混入日志，日志使用 stderr；非零退出始终失败。

stdout 上限 64 KiB，stderr 保留最后 64 KiB，默认超时 10 秒；超时或超量输出会终止 Unix 进程组，脚本不得脱离进程组。Windows 执行明确报错。--name 可绕过脚本，仍校验 pattern/排除项/冲突；--dry-run 会执行脚本但不由 rls 创建或推送 tag。无变更不调用脚本，全部 skip 返回 0。
