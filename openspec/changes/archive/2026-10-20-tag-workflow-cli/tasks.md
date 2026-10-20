## 1. Implementation

- [x] 实现分支消息及 tag/t 递增、预览、推送和重试。
- [x] 编写 Bash/Zsh/Fish 补全与安装目标。
- [x] 增加界面语言选择及消息目录。
- [x] 更新 README、skill、配置示例和产品纲要。
- [x] 英文化 Gherkin 与测试步骤，增加 bare remote 和 i18n 验收。

## 2. Verification

- [x] 运行 cargo test、Python 测试、fmt、Clippy、OpenSpec 严格校验。

验证结果：41 个验收场景、2 个 CLI i18n 测试、7 个 Python 测试通过；fmt、Clippy、OpenSpec 与 skill 校验通过。Bash 补全行为及 Bash/Zsh 语法已验证；环境无 Fish，未执行 Fish 运行时验证。
