use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum DbError {
    Connect(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(e) => write!(f, "DB 接続に失敗しました: {e}"),
            Self::Migrate(e) => write!(f, "マイグレーションに失敗しました: {e}"),
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connect(e) => Some(e),
            Self::Migrate(e) => Some(e),
        }
    }
}

pub async fn init_pool(db_path: &Path) -> Result<SqlitePool, DbError> {
    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .map_err(DbError::Connect)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(DbError::Migrate)?;
    Ok(pool)
}

#[cfg(test)]
pub async fn init_in_memory_pool() -> Result<SqlitePool, DbError> {
    use std::str::FromStr;
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .map_err(DbError::Connect)?
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(opts)
        .await
        .map_err(DbError::Connect)?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(DbError::Migrate)?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn init_in_memory_pool_succeeds_with_no_migrations() {
        let pool = init_in_memory_pool().await.unwrap();
        assert!(!pool.is_closed());
    }

    #[tokio::test]
    async fn pool_can_execute_simple_query() {
        let pool = init_in_memory_pool().await.unwrap();
        let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn init_pool_creates_db_file_if_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("db.sqlite");
        assert!(!path.exists());

        let pool = init_pool(&path).await.unwrap();
        assert!(path.exists());

        let row: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();
        assert_eq!(row.0, 1);
    }

    #[tokio::test]
    async fn migrations_create_all_expected_tables() {
        let pool = init_in_memory_pool().await.unwrap();
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master \
             WHERE type='table' AND name NOT LIKE '\\_%' ESCAPE '\\' \
             ORDER BY name",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let names: Vec<String> = rows.into_iter().map(|t| t.0).collect();
        assert_eq!(
            names,
            vec![
                "artifacts",
                "intake_reports",
                "messages",
                "phase_runs",
                "subtasks",
                "task_state_log",
                "tasks",
            ]
        );
    }

    #[tokio::test]
    async fn foreign_keys_pragma_is_enabled() {
        let pool = init_in_memory_pool().await.unwrap();
        let row: (i32,) = sqlx::query_as("PRAGMA foreign_keys")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 1);
    }
}
