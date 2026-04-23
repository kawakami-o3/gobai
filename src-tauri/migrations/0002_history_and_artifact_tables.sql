-- 履歴・本文参照テーブル: IntakeReport / Message / Artifact
-- intake レポートはサイズが小さく構造化されているため inline (TEXT JSON) で保持。
-- Message / Artifact は本文をファイル保存しパス参照する設計のため、NFR-8 由来の
-- truncated (容量上限到達) と content_purged_at (明示削除時刻) を持たせる。

CREATE TABLE intake_reports (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL UNIQUE REFERENCES tasks(id) ON DELETE CASCADE,
    clarifying_questions TEXT NOT NULL,    -- JSON 配列
    assumed_scope TEXT NOT NULL,           -- JSON {included, excluded, deferred}
    scope_warnings TEXT NOT NULL,          -- JSON 配列 (severity 付き)
    split_suggestions TEXT NOT NULL,       -- JSON 配列
    status TEXT NOT NULL                   -- draft | answered | approved
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    phase_run_id INTEGER NOT NULL REFERENCES phase_runs(id) ON DELETE CASCADE,
    role TEXT NOT NULL,                    -- prompt | response
    agent TEXT NOT NULL,                   -- codex | claude_code
    content_path TEXT NOT NULL,            -- マスキング済み本文ファイルへのパス
    tokens INTEGER,                        -- 計測できない場合 NULL
    duration_ms INTEGER,
    truncated INTEGER NOT NULL DEFAULT 0,
    content_purged_at TEXT
);
CREATE INDEX idx_messages_phase_run_id ON messages(phase_run_id);

CREATE TABLE artifacts (
    id INTEGER PRIMARY KEY,
    phase_run_id INTEGER NOT NULL REFERENCES phase_runs(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,                    -- design_doc | review_result | diff | log
    path TEXT NOT NULL,                    -- 本文ファイルへのパス (FR-12 / 03-04 docs 準拠)
    truncated INTEGER NOT NULL DEFAULT 0,
    content_purged_at TEXT
);
CREATE INDEX idx_artifacts_phase_run_id ON artifacts(phase_run_id);
