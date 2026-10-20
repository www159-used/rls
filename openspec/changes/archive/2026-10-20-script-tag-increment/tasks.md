## 1. Configuration and history
- [x] 增加 script 配置、参数数组和超时校验，验证模式/字段组合，保留 counter/manual 兼容性。
- [x] 实现有 tag_pattern 即祖先链、无 pattern 沿用数字规则，定义 glob 与 exclude_tags 优先级，不增加 tag_order。
- [x] 接入 msg，按所选分支 tag_prefix 生成 version 字段，不执行脚本。

## 2. Runner and tagging
- [x] 实现 v1 JSON 协议、严格响应校验和结构化错误。
- [x] 实现并发管道、有界输出、超时与 Unix 进程组终止；Windows 在 fetch 前拒绝需要调用脚本的分支。
- [x] 接入首次发布、无变更、dry-run、skip、--name 绕过与同名恢复及全部 skip 退出码 0。

## 3. Verification and documentation
- [x] 英文 Gherkin 覆盖脚本成功/失败/skip/非法 JSON/超时/超量输出。
- [x] 临时 Git 与 bare remote 覆盖自定义历史、多分支、冲突和推送恢复。
- [x] 验证 worktree 路径、当前脚本来源、目标 blob 读取、模式组合、全 skip、平台拒绝及 i18n 不改写名称。
- [x] 更新帮助、README、配置示例、补全与 skill 的实际能力说明。
- [x] 执行 fmt、Clippy、测试和 OpenSpec 严格校验后归档。

验证：68 个英文 Cucumber 场景、4 个 Rust 单元测试、2 个 CLI i18n 测试、7 个 Python 测试通过；fmt、Clippy、OpenSpec、skill 校验及 shell 示例执行通过。Unix 进程组回收已运行验证；Windows 分支通过 cfg 拒绝执行，未做 Windows 运行时验证。
