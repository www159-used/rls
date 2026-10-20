# language: zh-CN
@configuration-example
功能: 配置示例保持可用
  这是配置文档的检查，不代表发布流程已经实现；消息功能由独立场景验收。

  背景:
    假如仓库中的配置示例
    当按 TOML 读取示例

  场景大纲: 维护分支分别声明版本线
    那么分支 "<branch>" 的版本前缀为 "<prefix>"

    例子:
      | branch    | prefix |
      | 5.6       | 5.6.0  |
      | 5.6-2sync | 5.6.1  |

  场景: 首期后端和模板可供使用
    那么使用 git-tag 后端且消息模板存在
