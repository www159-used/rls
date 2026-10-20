## ADDED Requirements

### Requirement: Versioned message configuration

系统 SHALL 支持独立的 message.version 整数标记，当前格式为 1，省略时按 v1 读取。每个配置层 MUST 在合并前分别校验版本；不支持的版本报错并指出源文件，不修改配置。此字段与顶层 schema_version 和发布版本相互独立。

#### Scenario: Legacy compatibility
- **WHEN** 仓库省略 message.version，全局声明 version = 1
- **THEN** 两层均按 v1 解释并按既有规则合并

#### Scenario: Unknown version cannot be overridden
- **WHEN** 全局或仓库任一层使用不支持的消息版本
- **THEN** 返回非零状态并显示该配置路径与版本，不因其他层声明 v1 而忽略错误

#### Scenario: Invalid version type
- **WHEN** message.version 为字符串
- **THEN** 配置解析失败并指出来源
