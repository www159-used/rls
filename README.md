# rls

面向多分支、多技术栈项目的发布 CLI：生成发布计划、执行发布动作，再用 changelog 和模板生成可复制的消息。

状态：立项，尚未实现可执行命令。本文中的命令和配置是首期设计。

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

- [需求与验收条件](docs/PRD.md)
- [架构与扩展边界](docs/ARCHITECTURE.md)
- [实施任务](docs/ROADMAP.md)
- [配置草案](rls.example.toml)
- [消息模板示例](templates/release-message.md.j2)

计划使用 Rust 实现 CLI。第一版采用内置后端接口，不建设动态插件系统，也不提供默认发送聊天消息的功能。
