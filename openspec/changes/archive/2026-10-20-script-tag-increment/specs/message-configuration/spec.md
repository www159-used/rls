## MODIFIED Requirements

### Requirement: Branch message source

系统 SHALL 在 tag_pattern 存在时改用严格祖先链选择最近两个 tag；msg 不执行命名脚本。version SHALL 仅按所选分支的 tag_prefix 去前缀，不跨分支推断。

系统 SHALL 接受分支列表，默认当前分支；message/msg/m 等价。每个分支按仓库规则选取可达的最近两个 tag，数字规则按数值排序，pattern 规则按祖先链排序，读取其间全部提交并按参数顺序一次渲染。只读取本地 Git，不接受 tag 参数、range 或 --from，不保存 records。

#### Scenario: Multiple branches
- **WHEN** 用户选择两个有至少两个版本 tag 的分支
- **THEN** 合并各分支最近一次发布消息，不包含最新 tag 后的提交

#### Scenario: Invalid selection
- **WHEN** 任一分支缺失规则、少于两个 tag、存在数字歧义、非单调历史或浅克隆
- **THEN** 命令失败且不输出部分消息

#### Scenario: Detached checkout
- **WHEN** detached HEAD 下省略分支
- **THEN** 要求显式指定分支
