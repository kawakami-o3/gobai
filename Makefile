.DEFAULT_GOAL := help

.PHONY: help install dev build preview typecheck check-rust check \
        lint lint-rust lint-frontend \
        fmt fmt-rust fmt-frontend \
        fmt-check fmt-check-rust fmt-check-frontend \
        clean tauri cargo

help: ## このヘルプを表示
	@awk 'BEGIN{FS=":.*##"; printf "\nUsage: make <target>\n\nTargets:\n"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

install: ## bun install で依存をインストール
	bun install

dev: ## 動的ポートで tauri dev を起動 (scripts/dev.ts)
	bun scripts/dev.ts

build: ## tauri build で本番バンドルを作成 (Frontend + Rust + bundle)
	bun run tauri build

preview: ## vite preview で Frontend をプレビュー
	bun run preview

typecheck: ## TypeScript 型チェック (tsc --noEmit)
	bun run tsc --noEmit

check-rust: ## Rust コンパイルチェック (cargo check)
	cd src-tauri && cargo check --all-targets

check: typecheck check-rust ## typecheck + check-rust

lint-frontend: ## (placeholder) eslint 未設定
	@echo "[lint-frontend] eslint not configured yet (TODO: 01-project-setup/06-lint-format)"

lint-rust: ## clippy で Rust を lint (-D warnings)
	cd src-tauri && cargo clippy --all-targets --all-features -- -D warnings

lint: lint-frontend lint-rust ## frontend + rust の lint

fmt-frontend: ## (placeholder) prettier 未設定
	@echo "[fmt-frontend] prettier not configured yet (TODO: 01-project-setup/06-lint-format)"

fmt-rust: ## rustfmt で Rust を整形
	cd src-tauri && cargo fmt --all

fmt: fmt-frontend fmt-rust ## frontend + rust の整形

fmt-check-frontend: ## (placeholder) prettier --check 未設定
	@echo "[fmt-check-frontend] prettier not configured yet (TODO: 01-project-setup/06-lint-format)"

fmt-check-rust: ## rustfmt で整形差分を検査
	cd src-tauri && cargo fmt --all -- --check

fmt-check: fmt-check-frontend fmt-check-rust ## frontend + rust の整形検査

clean: ## dist と src-tauri/target を削除 (node_modules は残す)
	rm -rf dist
	cd src-tauri && cargo clean

tauri: ## tauri CLI passthrough (例: make tauri ARGS="info")
	bun run tauri $(ARGS)

cargo: ## cargo passthrough in src-tauri (例: make cargo ARGS="tree --depth 1")
	cd src-tauri && cargo $(ARGS)
