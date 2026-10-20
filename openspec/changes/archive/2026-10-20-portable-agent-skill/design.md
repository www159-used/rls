## Context

不同 Agent 使用相同的目录式 SKILL.md 结构，但扫描目录不同。项目的 OpenSpec 技能用于开发流程，不能代替面向 rls 用户的操作说明。

## Goals / Non-Goals

提供不依赖宿主专用工具、个人绝对路径或源码检出的 skill 副本。仅安装到用户选择的位置，不自动为所有 Agent 安装，不改变权限，不安装插件或启动进程。

## Decisions

- skills/rls 是唯一分发源，格式参考使用包内相对链接，并随包携带 AGPL v3 原文。
- 安装器使用 Python 标准库复制目录；默认 .agents/skills，可选择厂商目录、项目作用域或任意 skills 父目录。
- 目录存在时失败，显式 --force 才替换整个 rls 子目录；拒绝源目录重叠和目标符号链接。
- Makefile 包装安装参数，提供独立 test-skill，并在 make test 中包含安装器测试。
- skill 指导执行前查看本机帮助，明确当前只支持 message，以及其与 Rust Language Server 的同名冲突。

## Risks / Trade-offs

CLI 扫描路径可能随版本变化；README 引用官方路径文档并提供 SKILL_DIR 覆盖。更新采用复制，需要重新安装；FORCE=1 会丢弃安装副本中的本地修改，已在帮助中明确。不宣称已启动所有厂商 CLI 验证加载。
