# development-workflow Specification

## Purpose

建立规范驱动的变更流程与可执行的 Rust BDD 测试入口。此规范描述开发基础设施；发布功能仍以 openspec/project.md 为待实现目标。

## Requirements

### Requirement: OpenSpec change workflow

项目 SHALL 在 openspec/config.yaml 中提供产品上下文，并保留 Codex 的 OpenSpec 工作流技能。行为变更应先描述 proposal、specs、design 和 tasks，再实施和校验，完成后归档。

#### Scenario: Validate repository specifications
- **WHEN** 开发者运行 `openspec validate --all --strict --no-interactive`
- **THEN** 仓库中的规范与活动变更通过校验

### Requirement: Executable acceptance entry point

项目 SHALL 通过 `cargo test --locked --test acceptance` 执行 tests/features 下的中文 Gherkin 场景；步骤失败或未实现时返回失败状态。

#### Scenario: Check the configuration example
- **WHEN** 执行配置示例的 Cucumber 场景
- **THEN** 验证示例为合法 TOML、两条维护分支各自的版本前缀、git-tag 后端及消息模板文件存在
- **AND** 此结果仅表示配置示例与测试链路可用，不表示发布功能已实现

#### Scenario: Undefined step fails the suite
- **WHEN** 被执行的场景包含未定义的步骤
- **THEN** 测试进程以非零状态退出
