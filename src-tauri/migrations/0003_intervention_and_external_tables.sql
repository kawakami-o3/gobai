-- 介入・外部テーブル: UserDecision / Worktree / RepoSummary
-- worktrees は新モデル (subtask_id を持たない、1 タスク 1 件) を反映する。
-- payload や source 等の値域は SQL CHECK ではなく Rust 側 enum で型ガードする方針。

CREATE TABLE user_decisions (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    subtask_id INTEGER REFERENCES subtasks(id) ON DELETE CASCADE,        -- intake/abort 等は NULL
    phase_run_id INTEGER REFERENCES phase_runs(id) ON DELETE CASCADE,    -- phase 外の判断は NULL
    kind TEXT NOT NULL,                    -- approve_intake | approve_decomp | answer_unknowns | loop_extend | allow_critical | abort | split_task | edit_decomp | resume_from_force_pause | ...
    payload TEXT NOT NULL,                 -- JSON (kind ごとに形状が異なる)
    decided_at TEXT NOT NULL               -- ISO 8601
);
CREATE INDEX idx_user_decisions_task_id ON user_decisions(task_id);
CREATE INDEX idx_user_decisions_subtask_id ON user_decisions(subtask_id);
CREATE INDEX idx_user_decisions_phase_run_id ON user_decisions(phase_run_id);

CREATE TABLE worktrees (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL UNIQUE REFERENCES tasks(id) ON DELETE CASCADE,  -- 1 タスク 1 件
    path TEXT NOT NULL,                    -- worktree のファイルシステム上のパス
    branch TEXT NOT NULL,                  -- 紐付くブランチ名
    created_at TEXT NOT NULL,
    removed_at TEXT                        -- cleanup 後 NULL でなくなる (soft delete)
);

CREATE TABLE repo_summaries (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    generated_at TEXT NOT NULL,            -- ISO 8601
    generated_at_commit TEXT NOT NULL,     -- delta 算出のための base commit SHA
    content_path TEXT NOT NULL,            -- 本文ファイルへのパス
    source TEXT NOT NULL                   -- initial | post_intake | pre_design | manual_refresh
);
CREATE INDEX idx_repo_summaries_task_id ON repo_summaries(task_id);
