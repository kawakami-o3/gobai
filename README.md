# gobai

codex CLI と claude code CLI を「実装役」と「批評役」として交互に組み合わせる、ユーザー伴走型コーディングエージェントの Tauri デスクトップアプリ。

## ドキュメント

設計・要件・タスク分解は別リポジトリ `../gobai-planning/` に集約されています。

- `../gobai-planning/docs/` — プロジェクト概要 / ユースケース / 要件 / アーキテクチャ / ADR
- `../gobai-planning/tasks/` — 大項目別タスク (`000-overview.md` が入口)

## 開発環境

- パッケージマネージャ: **bun** を使用 (`bun install` / `bun run ...`)
- 対象 OS: macOS arm64 / Windows x64 (詳細は後続タスク `01-project-setup/07-os-setup-guide` で整備予定)

## 起動コマンド (placeholder)

> 詳細な dev / build コマンドは `01-project-setup/05-dev-scripts` で整備予定。現時点では以下を参照。

```sh
make help    # 利用可能なターゲット一覧
make install # 依存をインストール
make dev     # tauri dev (動的ポート選択)
```

## ライセンス

[MIT License](./LICENSE)
