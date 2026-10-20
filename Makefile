.DEFAULT_GOAL := help

CARGO ?= cargo
PREFIX ?= $(HOME)/.local
PYTHON ?= python3
AGENT ?= common
SCOPE ?= user
PROJECT ?= $(CURDIR)
SKILL_DIR ?=
FORCE ?= 0
LEVEL ?= patch
CARGO_RELEASE_VERSION := 1.1.2

.PHONY: help install install-skill test-skill build test check setup-release release release-execute

help:
	@printf '%s\n' \
	  'make install                安装或更新到 ~/.local/bin' \
	  'make install PREFIX=/path   安装到 /path/bin' \
	  'make install-completions    安装 Bash/Zsh/Fish 补全脚本' \
	  'make install-skill          安装通用 skill 到 ~/.agents/skills' \
	  'make install-skill AGENT=claude  使用指定 Agent 的 skills 目录' \
	  'make test-skill             验证 skill 安装器' \
	  'make build                  构建 release 版本' \
	  'make setup-release          安装 cargo-release 1.1.2' \
	  'make release LEVEL=patch    预览版本变更，不打 tag、不推送' \
	  'make release-execute LEVEL=patch  更新版本、创建 tag 并推送 origin，保留确认提示' \
	  'make test                   运行测试' \
	  'make check                  检查格式、Clippy 和 OpenSpec'

install:
	$(CARGO) install --path . --locked --root "$(PREFIX)"
	"$(PREFIX)/bin/rls" --version
	@printf '%s\n' '安装完成。请确保 $(PREFIX)/bin 位于 PATH 前面，避免调用 Rust 工具链的同名 rls。'

build:
	$(CARGO) build --locked --release

setup-release:
	$(CARGO) install cargo-release --version "$(CARGO_RELEASE_VERSION)" --locked

release:
	$(CARGO) release --isolated --config release.toml "$(LEVEL)"

release-execute:
	$(CARGO) release --isolated --config release.toml "$(LEVEL)" --execute

install-skill:
	$(PYTHON) scripts/install_skill.py --agent "$(AGENT)" --scope "$(SCOPE)" --project "$(PROJECT)" $(if $(strip $(SKILL_DIR)),--dest "$(SKILL_DIR)") $(if $(filter 1,$(FORCE)),--force)

test-skill:
	$(PYTHON) -m unittest discover -s scripts -p 'test_*.py'

test: test-skill
	$(CARGO) test --locked

check:
	$(CARGO) fmt --check
	$(CARGO) clippy --locked --all-targets -- -D warnings
	openspec validate --all --strict --no-interactive

.PHONY: install-completions
install-completions:
	install -d "$(PREFIX)/share/bash-completion/completions" "$(PREFIX)/share/zsh/site-functions" "$(PREFIX)/share/fish/vendor_completions.d"
	install -m 644 completions/rls.bash "$(PREFIX)/share/bash-completion/completions/rls"
	install -m 644 completions/_rls "$(PREFIX)/share/zsh/site-functions/_rls"
	install -m 644 completions/rls.fish "$(PREFIX)/share/fish/vendor_completions.d/rls.fish"
