# language: zh-CN
功能: 从 Git tags 按分层配置生成发布消息
  背景:
    假如隔离的仓库、全局目录和 Git tags

  场景: 无配置使用内置模板
    当生成消息
    那么消息包含区间内全部提交且保持顺序和换行
    并且工作区和 Git 引用保持不变且未生成 records

  场景: 仓库外明确报错
    假如从仓库外执行
    当生成消息
    那么错误包含 "请在目标 Git 仓库"

  场景: 在子目录读取 Git 区间
    假如从仓库子目录执行
    当生成消息
    那么消息包含区间内全部提交且保持顺序和换行

  场景: 输出文件保留换行
    当将消息写入文件
    那么工作区和 Git 引用保持不变且未生成 records

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
    并且工作区和 Git 引用保持不变且未生成 records
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

  场景: 根据版本线按数字选择上一 tag 并排除无关 tag
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "9.0"
      tag_prefix = "v"
      """
    并且tag "v9.0.2" 指向 "HEAD~2"
    并且tag "v9.0.9" 指向 "HEAD~1"
    并且tag "v9.0.10" 指向 "HEAD"
    并且tag "v2.0.99" 指向 "HEAD~1"
    并且tag "2026-10-20" 指向 "HEAD~1"
    并且存在不相关分支上的同版本线 tag
    当执行消息命令 "message v9.0.10"
    那么消息包含 "v9.0.10"
    并且消息包含 "修复二"
    并且消息不包含 "修复一"

  场景: 自动基线与显式基线结果一致
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    当执行消息命令 "msg v1.0.1"
    那么消息包含区间内全部提交且保持顺序和换行
    并且工作区和 Git 引用保持不变且未生成 records

  场景: 没有规则时要求显式基线
    当执行消息命令 "message v1.0.1"
    那么错误包含 "请配置 branches 或使用 --from"

  场景: 没有上一 tag 时不猜测
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    当执行消息命令 "message v1.0.0"
    那么错误包含 "没有上一 tag"

  场景大纲: 显式区间必须有效
    当执行消息命令 "m <target> --from <from>"
    那么错误包含 "<error>"

    例子:
      | target  | from     | error                    |
      | v1.0.1  | missing  | 找不到指向提交的 tag     |
      | v1.0.1  | v1.0.1   | 无新变更                 |
      | v1.0.0  | v1.0.1   | 不是目标                 |
      | HEAD    | v1.0.0   | 找不到指向提交的 tag     |
      | v1.0.1  | HEAD~1   | 非法 tag                 |

  场景: 历史版本倒退时报错
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    并且tag "v1.0.9" 指向 "HEAD~1"
    当执行消息命令 "message v1.0.1"
    那么错误包含 "版本历史不单调"

  场景: --from 不能应用于多个目标
    当执行消息命令 "message v1.0.0 v1.0.1 --from v1.0.0"
    那么错误包含 "--from 仅适用于一个目标 tag"

  场景: 多版本线分别选择基线
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      [branches.maintenance]
      version_prefix = "2.0"
      tag_prefix = "v"
      """
    并且tag "v2.0.0" 指向 "HEAD~1"
    并且tag "v2.0.1" 指向 "HEAD"
    当执行消息命令 "message v1.0.1 v2.0.1"
    那么消息包含 "v1.0.1"
    并且消息包含 "v2.0.1"

  场景: 不再支持 record 参数
    当执行消息命令 "message --record example.json"
    那么错误包含 "--record"

  场景: 同一数字的多个 tag 不自动选择
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    并且tag "v1.0.00" 指向 "HEAD~2"
    当执行消息命令 "message v1.0.1"
    那么错误包含 "版本基线有歧义"

  场景: 浅克隆不自动猜测基线
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    并且从深度一的浅克隆执行
    当执行消息命令 "message v1.0.1"
    那么错误包含 "浅克隆无法可靠选择上一 tag"

  场景: 全局版本线不影响仓库
    假如全局配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    当执行消息命令 "message v1.0.1"
    那么错误包含 "请配置 branches 或使用 --from"

  场景: 多目标任一失败时不输出半条消息
    假如仓库配置为
      """
      schema_version = 1
      [branches.main]
      version_prefix = "1.0"
      tag_prefix = "v"
      """
    当执行消息命令 "message v1.0.1 missing"
    那么错误包含 "找不到指向提交的 tag"
