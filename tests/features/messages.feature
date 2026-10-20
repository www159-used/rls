# language: zh-CN
功能: 按分层配置重新生成发布消息
  背景:
    假如隔离的仓库、全局目录和发布记录

  场景: 无配置使用内置模板
    当生成消息
    那么消息只包含成功项且保持提交顺序和换行
    并且发布记录和 Git 引用保持不变

  场景: 仓库外也可用显式记录
    假如从仓库外执行
    当生成消息
    那么消息只包含成功项且保持提交顺序和换行

  场景: 在子目录通过 ID 读取记录
    假如从仓库子目录执行
    当按记录 ID 生成消息
    那么消息只包含成功项且保持提交顺序和换行

  场景: 输出文件保留换行
    当将消息写入文件
    那么发布记录和 Git 引用保持不变

  场景: 输出不能覆盖输入记录
    当试图用输出覆盖发布记录
    那么错误包含 "输出文件不能覆盖发布记录"
    并且发布记录和 Git 引用保持不变

  场景: 全局模板与变量被仓库继承
    假如全局配置为
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      [message.variables]
      greeting = "全局称呼"
      suffix = "谢谢"
      """
    并且全局模板为
      """
      {{ variables.greeting }}|{{ variables.suffix }}|{{ releases[0].tag }}
      """
    当生成消息
    那么消息包含 "全局称呼|谢谢|v1.0.1"
    假如仓库配置为
      """
      schema_version = 1
      [message.variables]
      greeting = "仓库称呼"
      releases = "不能覆盖事实"
      """
    当生成消息
    那么消息包含 "仓库称呼|谢谢|v1.0.1"
    并且发布记录和 Git 引用保持不变
    假如仓库配置为
      """
      schema_version = 1
      [message.variables]
      greeting = ""
      """
    当生成消息
    那么消息包含 "|谢谢|v1.0.1"

  场景: linked worktree 读取共享元目录和仓库相对模板
    假如全局配置为
      """
      schema_version = 1
      [message.variables]
      greeting = "继承变量"
      """
    并且仓库配置为
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    并且仓库模板为
      """
      仓库模板|{{ variables.greeting }}|{{ releases[0].tag }}
      """
    并且从 linked worktree 执行
    当生成消息
    那么消息包含 "仓库模板|继承变量|v1.0.1"

  场景: 仓库模板覆盖全局模板
    假如全局配置为
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    并且全局模板为
      """
      全局模板
      """
    并且仓库配置为
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    并且仓库模板为
      """
      仓库模板
      """
    当生成消息
    那么消息包含 "仓库模板"

  场景: 显式模板丢失时报错
    假如仓库配置为
      """
      schema_version = 1
      [message]
      template = "missing.j2"
      """
    当生成消息
    那么错误包含 "missing.j2"

  场景: 模板变量拼错时报错
    假如仓库配置为
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    并且仓库模板为
      """
      {{ variables.typo }}
      """
    当生成消息
    那么错误包含 "undefined"

  场景: 模板语法错误时报错
    假如仓库配置为
      """
      schema_version = 1
      [message]
      template = "templates/message.j2"
      """
    并且仓库模板为
      """
      {% broken %}
      """
    当生成消息
    那么错误包含 "解析消息模板"

  场景大纲: 非法配置不能静默回退
    假如仓库配置为
      """
      <config>
      """
    当生成消息
    那么错误包含 "<error>"

    例子:
      | config             | error          |
      | schema_version = 9 | schema_version |
      | invalid = [        | 解析配置       |

  场景: 全局配置损坏也不能静默回退
    假如全局配置为
      """
      schema_version = 99
      """
    当生成消息
    那么错误包含 "schema_version 99"

  场景: 未知记录版本报错
    假如发布记录 schema 为 99
    当生成消息
    那么错误包含 "不支持发布记录 schema_version 99"

  场景: 绝对模板路径保持不变
    假如仓库模板使用绝对路径
    当生成消息
    那么消息包含 "绝对模板|v1.0.1"

  场景: 显式消息版本与旧配置合并
    假如全局配置为
      """
      schema_version = 1
      [message]
      version = 1
      template = "templates/message.j2"
      [message.variables]
      greeting = "全局"
      """
    并且全局模板为
      """
      {{ variables.greeting }}|{{ releases[0].tag }}
      """
    并且仓库配置为
      """
      schema_version = 1
      [message.variables]
      greeting = "旧仓库配置"
      """
    当生成消息
    那么消息包含 "旧仓库配置|v1.0.1"

  场景大纲: 各层独立拒绝不支持的消息版本
    假如全局配置为
      """
      schema_version = 1
      [message]
      version = <global_version>
      """
    并且仓库配置为
      """
      schema_version = 1
      [message]
      version = <project_version>
      """
    当生成消息
    那么错误包含 "不支持 message.version <invalid_version>"

    例子:
      | global_version | project_version | invalid_version |
      | 2              | 1               | 2               |
      | 1              | 2               | 2               |
      | 1              | 0               | 0               |

  场景: 消息版本必须为整数
    假如仓库配置为
      """
      schema_version = 1
      [message]
      version = "1"
      """
    当生成消息
    那么错误包含 "解析配置"
