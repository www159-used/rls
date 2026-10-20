#!/usr/bin/env python3
"""Install the portable rls skill without depending on an agent's CLI."""

import argparse
import os
from pathlib import Path
import shutil
import sys

SOURCE = Path(__file__).resolve().parent.parent / "skills" / "rls"


def skills_root(agent, scope, project):
    if scope == "project":
        folder = {
            "common": ".agents", "codex": ".agents", "claude": ".claude",
            "gemini": ".gemini", "opencode": ".opencode",
        }[agent]
        return project / folder / "skills"
    home = Path.home()
    if agent == "opencode":
        return Path(os.environ.get("XDG_CONFIG_HOME") or home / ".config") / "opencode" / "skills"
    folder = {
        "common": ".agents", "codex": ".agents", "claude": ".claude", "gemini": ".gemini",
    }[agent]
    return home / folder / "skills"


def install(root, force=False):
    destination = root.expanduser().absolute() / "rls"
    if destination.is_symlink():
        raise ValueError(f"目标是符号链接，请选择实际安装目录: {destination}")
    source = SOURCE.resolve()
    resolved = destination.resolve()
    if resolved == source or source in resolved.parents or resolved in source.parents:
        raise ValueError("安装目录不能与 skill 源码目录重叠")
    if destination.exists():
        if not destination.is_dir():
            raise ValueError(f"目标不是目录: {destination}")
        if not force:
            raise ValueError(f"目标已存在: {destination}；更新时使用 --force（覆盖整个 rls skill 目录）")
        shutil.rmtree(destination)
    root.expanduser().mkdir(parents=True, exist_ok=True)
    shutil.copytree(SOURCE, destination)
    return destination


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--agent", choices=["common", "codex", "claude", "gemini", "opencode"], default="common")
    parser.add_argument("--scope", choices=["user", "project"], default="user")
    parser.add_argument("--project", type=Path, default=Path.cwd(), help="项目级安装的目标仓库")
    parser.add_argument("--dest", type=Path, help="自定义 skills 父目录，覆盖 agent/scope 路径")
    parser.add_argument("--force", action="store_true", help="替换已有 rls skill 目录")
    args = parser.parse_args()
    root = args.dest if args.dest is not None else skills_root(args.agent, args.scope, args.project.expanduser())
    try:
        destination = install(root, args.force)
    except (OSError, ValueError) as error:
        print(f"安装失败: {error}", file=sys.stderr)
        return 1
    print(f"已安装: {destination}")
    print("在 Agent 中重新加载 skills 或重启会话。此操作仅安装 skill，不安装 rls 可执行文件。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
