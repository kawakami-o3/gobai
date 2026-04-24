//! Task の `status` 変更を 1 トランザクション内で行い、同時に
//! `task_state_log` に履歴を残すための単一エントリポイント。直接
//! `UPDATE tasks SET status = ...` を呼ぶ箇所を散在させると履歴記録が
//! 抜けるため、状態遷移は本モジュールの関数経由に集約する。
//!
//! 並行書き込みが本格化したら SQLITE_BUSY 対策 (WAL モード /
//! busy_timeout 設定 / `BEGIN IMMEDIATE`) を別途検討する。現状は
//! 単一書き手の前提で `BEGIN DEFERRED` (sqlx の既定) のまま運用する。

use sqlx::SqlitePool;
use std::fmt;

#[derive(Debug)]
pub enum TransitionError {
    /// 指定 ID のタスクが存在しなかった。
    TaskNotFound(i64),
    /// 下層 sqlx 由来のエラーをそのまま伝える。
    Sqlx(sqlx::Error),
}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TaskNotFound(id) => write!(f, "タスクが見つかりません (id={id})"),
            Self::Sqlx(e) => write!(f, "状態遷移の永続化に失敗しました: {e}"),
        }
    }
}

impl std::error::Error for TransitionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TaskNotFound(_) => None,
            Self::Sqlx(e) => Some(e),
        }
    }
}

impl From<sqlx::Error> for TransitionError {
    fn from(e: sqlx::Error) -> Self {
        Self::Sqlx(e)
    }
}

/// `tasks.status` を `new_state` に書き換えると同時に `task_state_log`
/// へ遷移履歴を 1 件追記する。両者は単一トランザクション内で実行され、
/// どちらかが失敗すれば全体がロールバックされる。
///
/// `reason` には遷移の動機 (例: "user_run", "intake_done") を記録する。
/// 同状態への遷移 (`pending → pending` 等) も拒否せず履歴に残す方針。
/// 呼び出し側で no-op を弾く必要がある場合はその場で判定すること。
pub async fn transition_task_state(
    pool: &SqlitePool,
    task_id: i64,
    new_state: &str,
    reason: &str,
) -> Result<(), TransitionError> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

    let mut tx = pool.begin().await?;

    let row: Option<(String,)> = sqlx::query_as("SELECT status FROM tasks WHERE id = ?")
        .bind(task_id)
        .fetch_optional(&mut *tx)
        .await?;
    let old_state = row
        .map(|(s,)| s)
        .ok_or(TransitionError::TaskNotFound(task_id))?;

    sqlx::query("UPDATE tasks SET status = ?, updated_at = ? WHERE id = ?")
        .bind(new_state)
        .bind(&now)
        .bind(task_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO task_state_log (task_id, old_state, new_state, transition_reason, occurred_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(task_id)
    .bind(&old_state)
    .bind(new_state)
    .bind(reason)
    .bind(&now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_in_memory_pool;

    const FIXED_TS: &str = "2026-04-24T00:00:00Z";

    async fn insert_task(pool: &SqlitePool, status: &str) -> i64 {
        let result = sqlx::query(
            "INSERT INTO tasks (title, body, repo_path, kind, confirm_level, minor_policy, status, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind("テストタスク")
        .bind("body")
        .bind("/tmp/repo")
        .bind("new_feature")
        .bind("normal")
        .bind("record_and_continue")
        .bind(status)
        .bind(FIXED_TS)
        .bind(FIXED_TS)
        .execute(pool)
        .await
        .unwrap();
        result.last_insert_rowid()
    }

    #[tokio::test]
    async fn transition_updates_status_and_logs() {
        let pool = init_in_memory_pool().await.unwrap();
        let task_id = insert_task(&pool, "pending").await;

        transition_task_state(&pool, task_id, "intake", "user_run")
            .await
            .unwrap();

        let (status,): (String,) = sqlx::query_as("SELECT status FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(status, "intake");

        let logs: Vec<(Option<String>, String, String)> = sqlx::query_as(
            "SELECT old_state, new_state, transition_reason FROM task_state_log \
             WHERE task_id = ? ORDER BY id",
        )
        .bind(task_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(
            logs[0],
            (
                Some("pending".to_string()),
                "intake".to_string(),
                "user_run".to_string()
            )
        );
    }

    #[tokio::test]
    async fn missing_task_returns_error() {
        let pool = init_in_memory_pool().await.unwrap();
        let result = transition_task_state(&pool, 999, "intake", "user_run").await;
        assert!(matches!(result, Err(TransitionError::TaskNotFound(999))));
    }

    #[tokio::test]
    async fn consecutive_transitions_chain_old_state() {
        let pool = init_in_memory_pool().await.unwrap();
        let task_id = insert_task(&pool, "pending").await;

        transition_task_state(&pool, task_id, "intake", "r1")
            .await
            .unwrap();
        transition_task_state(&pool, task_id, "waiting_intake", "r2")
            .await
            .unwrap();

        let logs: Vec<(Option<String>, String)> = sqlx::query_as(
            "SELECT old_state, new_state FROM task_state_log \
             WHERE task_id = ? ORDER BY id",
        )
        .bind(task_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0], (Some("pending".to_string()), "intake".to_string()));
        assert_eq!(
            logs[1],
            (Some("intake".to_string()), "waiting_intake".to_string())
        );
    }

    #[tokio::test]
    async fn updated_at_is_advanced() {
        let pool = init_in_memory_pool().await.unwrap();
        let task_id = insert_task(&pool, "pending").await;

        transition_task_state(&pool, task_id, "intake", "r")
            .await
            .unwrap();

        let (updated_at,): (String,) = sqlx::query_as("SELECT updated_at FROM tasks WHERE id = ?")
            .bind(task_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_ne!(updated_at, FIXED_TS);
    }

    #[tokio::test]
    async fn same_state_transition_is_logged() {
        let pool = init_in_memory_pool().await.unwrap();
        let task_id = insert_task(&pool, "pending").await;

        transition_task_state(&pool, task_id, "pending", "noop_replay")
            .await
            .unwrap();

        let logs: Vec<(Option<String>, String, String)> = sqlx::query_as(
            "SELECT old_state, new_state, transition_reason FROM task_state_log \
             WHERE task_id = ? ORDER BY id",
        )
        .bind(task_id)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(
            logs[0],
            (
                Some("pending".to_string()),
                "pending".to_string(),
                "noop_replay".to_string()
            )
        );
    }
}
