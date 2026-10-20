## Context

Git 是版本和提交事实来源，用户只维护元目录内的配置。仓库级版本规则已经在示例中定义，现在由消息流程实际消费。

## Goals / Non-Goals

实现现有 tag 的显式/自动区间与渲染，不实现 tag 创建、远端发布、自动 fetch 或 nexttag。

## Decisions

- 输入限制为 refs/tags 下的有效名称，解析 annotated/lightweight tag 到提交 SHA 后构造 Git log 参数，避免 revision 表达式歧义。
- 自动模式匹配 tag_prefix + version_prefix + 数字后缀。仅接受目标祖先中的候选，按 u64 数字排序；首次、相同数字、历史倒序或不在线性祖先链上的候选要求 --from。
- 浅克隆不能保证自动基线完整，要求补全历史或显式指定可验证基线。
- 显式 --from 只允许一个目标，验证祖先关系和非空区间；无规则时 branch 为空、version 保留原 tag。
- --reverse --topo-order 输出完整提交列表，保留 SHA 和 subject，包括 merge commits。Git replace objects 禁用，文本由 Git 输出 UTF-8。
- 先解析所有目标、再渲染和输出，任一目标失败不输出半条消息。
- 配置继承与 message.version 不变，branches 只读取仓库配置。输出层保留文件和剪贴板行为。

## Risks / Trade-offs

本地 tag 不证明远端发布成功。用户需要的远端 tag 若尚未存在本地，应单独获取。按 tag 历史生成的消息会随着 tag 被移动而改变；本次按用户要求不引入快照记录。

## Migration Plan

旧的 rls message <record-id> 改为 rls message <tag> --from <previous-tag>，配置版本线后可省略 --from。旧 records 不再读取，不删除用户现有记录文件。同步更新 skill，已安装副本需重新安装。
