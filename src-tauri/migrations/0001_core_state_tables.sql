-- コア状態テーブル: Task / TaskStateLog / SubTask / PhaseRun
-- 状態識別子は ADR-001 §3 (Task) と §4 (SubTask) に準拠する。
-- TaskStateLog は §6 のトランザクション付き履歴要件のための土台。
-- 状態文字列の値域は SQL の CHECK 制約ではなく Rust 側の enum で型ガードする。

CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    body TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    kind TEXT NOT NULL,                       -- new_feature | bugfix | refactor
    confirm_level TEXT NOT NULL,              -- strict | normal | autonomous
    minor_policy TEXT NOT NULL,               -- record_and_continue | block_until_resolved | prompt_user
    status TEXT NOT NULL,                     -- ADR-001 §3
    force_paused INTEGER NOT NULL DEFAULT 0,  -- bool (0/1)
    api_call_count INTEGER NOT NULL DEFAULT 0,
    token_estimate INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,                 -- ISO 8601
    updated_at TEXT NOT NULL
);

CREATE TABLE task_state_log (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    old_state TEXT,                           -- 初回挿入時は NULL の可能性あり
    new_state TEXT NOT NULL,
    transition_reason TEXT NOT NULL,
    occurred_at TEXT NOT NULL
);
CREATE INDEX idx_task_state_log_task_id ON task_state_log(task_id);

CREATE TABLE subtasks (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    "order" INTEGER NOT NULL,                 -- 表示順 (SQL 予約語のためクオート)
    title TEXT NOT NULL,
    intent TEXT NOT NULL,
    preconditions TEXT NOT NULL,              -- JSON
    dependencies TEXT NOT NULL,               -- JSON: 他 subtask id 配列
    related_files TEXT NOT NULL,              -- JSON
    unknowns TEXT NOT NULL,                   -- JSON
    acceptance_criteria TEXT NOT NULL,        -- JSON
    status TEXT NOT NULL                      -- ADR-001 §4
);
CREATE INDEX idx_subtasks_task_id ON subtasks(task_id);

CREATE TABLE phase_runs (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    subtask_id INTEGER REFERENCES subtasks(id) ON DELETE CASCADE,  -- intake/decompose は NULL
    phase TEXT NOT NULL,                      -- intake | decompose | design | review_design | impl | review_impl
    iteration INTEGER NOT NULL,
    status TEXT NOT NULL,
    started_at TEXT NOT NULL,
    finished_at TEXT,                         -- 実行中は NULL
    commit_sha TEXT                           -- impl/review_impl のみ
);
CREATE INDEX idx_phase_runs_task_id ON phase_runs(task_id);
CREATE INDEX idx_phase_runs_subtask_id ON phase_runs(subtask_id);
