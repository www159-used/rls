## Context

Git 是发布事实来源，配置仅维护规则与模板，不引入 records。

## Goals / Non-Goals

支持多分支轻量 tag 和消息、显式推送、可重试失败；不实现制品发布或持久化 plan/run。

## Decisions

分支列表保持输入顺序；消息一次渲染，tag 逐项执行。tag 使用 fetch 后的远端 SHA，创建前再次核对远端，update-ref 比较创建避免覆盖。数字版本按祖先关系验证。推送只带所选 tag refspec，重试用单分支 --name。补全只读，翻译在格式化边界进行，不替换用户数据。测试使用英文，i18n 用独立进程验证 locale。

## Risks / Trade-offs

旧 tag 参数是破坏性迁移；README 与 skill 改为分支。批量不承诺事务；成功项保留。dry-run fetch 会更新本地引用。远端分支在最终核对后仍可能变化，tag 固定指向已核对的 SHA。
