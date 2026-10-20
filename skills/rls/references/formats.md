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

[branches."5.6-2sync"]
version_prefix = "5.6.1"
tag_prefix = "v"
increment = "counter"
```

这些规则只读取仓库配置，不从全局继承。第一条匹配 `v5.6.0.<非负整数>`。msg 选择分支可达的同线 tag，按数字排序取最近两个；历史倒退、数字重复或分叉歧义时报错，不接受 --from。version 去除 tag_prefix，branch 为请求的配置键。

`increment` 支持 counter（默认，末位加一）或 manual（tag 命令必须指定 --name）。仓库 `[release] remote = "origin"` 指定远端，命令 -r 可覆盖；推送仅由 -p/--push 启用。顶层 schema_version 仍为 1，新字段省略时保持默认。

## 模板数据

每项 release 提供：

| 字段 | 含义 |
|---|---|
| tag / previous_tag | 目标及基线 tag 名 |
| sha / previous_sha | 解析后的提交 SHA |
| version | 去掉匹配规则 tag_prefix 的版本， |
| branch | 匹配的仓库版本线配置键 |
| commits | 基线之后直到目标的全部提交，每条包含 sha、subject |

`releases` 按命令参数顺序排列，commits 使用反向拓扑顺序，包含合并提交。不提供 release_id 或 status，也不保存 records。消息使用的 tag 必须在当前仓库内；linked worktree 共用 Git 引用和配置。
