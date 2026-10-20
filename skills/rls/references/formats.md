# 配置与发布记录

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

模板使用 MiniJinja。可访问 `release_id`、`variables`、`releases`；循环和条件可用，未定义变量严格报错，保留模板尾部换行。模板可以读取每条发布的 branch、version、tag、sha、status、commits。

## 发布记录 schema v1

以下仅为结构示例，不能作为真实发布凭据。以实际保存的记录为准。

```json
{
  "schema_version": 1,
  "release_id": "example-release",
  "releases": [
    {
      "status": "success",
      "branch": "main",
      "version": "1.0.1",
      "tag": "v1.0.1",
      "sha": "1111111111111111111111111111111111111111",
      "commits": [
        {
          "sha": "1111111111111111111111111111111111111111",
          "subject": "修复示例问题"
        }
      ]
    }
  ]
}
```

上述字段均需提供。`status` 必须为 `success`、`failed` 或 `skipped`；只有 success 进入模板。记录 ID 不可为空；按位置参数读取时 ID 仅支持 ASCII 字母、数字、连字符和下划线，文件内 release_id 必须与参数一致。

`releases` 和每项的 `commits` 按记录顺序传给模板，不重新访问发布后端。仓库配置跨 worktree 共享，但 `.rls/state/` 是当前 worktree 的本地状态；另一 worktree 的记录需显式指定 `--record`。
