## Context

现有顶层 schema_version 描述整个配置文件，不能独立表达消息格式演进。

## Goals / Non-Goals

支持 message.version 和旧配置兼容。不改变发布版本、CLI 参数或记录格式。

## Decisions

message.version 是每个源文件的格式标记，非可继承的用户设置。缺失时为 v1；显式值必须为整数 1。全局和仓库分别校验后才合并，未知版本不能被另一层覆盖。未来增加格式时先迁移各层到内部表示再合并，不修改源文件。

## Risks / Trade-offs

为兼容旧配置，缺省将始终代表 v1，不随未来默认格式升级。
