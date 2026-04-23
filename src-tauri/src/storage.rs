use serde::Serialize;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tauri::Manager;

const CONTENT_DIR_NAME: &str = "content";
const DB_FILENAME: &str = "db.sqlite";

/// Filesystem layout for the storage layer.
///
/// `db` is the path the SQLite file *will* live at; this module does not
/// create the file (that's sqlx's job in a later step). Only the parent
/// directory and the body-content subdirectory are created here.
#[derive(Debug, Clone, Serialize)]
pub struct StoragePaths {
    pub root: PathBuf,
    pub db: PathBuf,
    pub content_dir: PathBuf,
}

#[derive(Debug)]
pub enum StorageError {
    MissingDataDir,
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDataDir => {
                write!(f, "アプリのデータディレクトリを取得できませんでした")
            }
            Self::Io { path, source } => {
                write!(
                    f,
                    "ストレージ初期化に失敗しました ({}): {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::MissingDataDir => None,
        }
    }
}

/// Pure helper: create root + content/ under `root`, set Unix-only 0o700.
/// `db` path is computed but not created.
pub fn init_storage_at(root: &Path) -> Result<StoragePaths, StorageError> {
    create_dir(root)?;
    let content_dir = root.join(CONTENT_DIR_NAME);
    create_dir(&content_dir)?;

    #[cfg(unix)]
    {
        set_owner_only(root)?;
        set_owner_only(&content_dir)?;
    }

    Ok(StoragePaths {
        root: root.to_path_buf(),
        db: root.join(DB_FILENAME),
        content_dir,
    })
}

/// Tauri-integrated entry: resolves `app_local_data_dir` then delegates.
pub fn init_storage(app: &tauri::AppHandle) -> Result<StoragePaths, StorageError> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|_| StorageError::MissingDataDir)?;
    init_storage_at(&root)
}

fn create_dir(path: &Path) -> Result<(), StorageError> {
    fs::create_dir_all(path).map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(unix)]
fn set_owner_only(path: &Path) -> Result<(), StorageError> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|e| StorageError::Io {
        path: path.to_path_buf(),
        source: e,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_storage_at_creates_root_and_content() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("gobai");
        let p = init_storage_at(&root).unwrap();

        assert!(root.is_dir());
        assert!(p.content_dir.is_dir());
        assert_eq!(p.content_dir, root.join(CONTENT_DIR_NAME));
        assert_eq!(p.db, root.join(DB_FILENAME));
        assert_eq!(p.root, root);
    }

    #[test]
    fn init_storage_at_does_not_create_db_file() {
        let tmp = tempfile::tempdir().unwrap();
        let p = init_storage_at(tmp.path()).unwrap();

        // db ファイルの作成は sqlx の責務。ここではパスを返すだけ。
        assert!(!p.db.exists());
    }

    #[test]
    fn init_storage_at_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let p1 = init_storage_at(tmp.path()).unwrap();
        let p2 = init_storage_at(tmp.path()).unwrap();
        assert_eq!(p1.root, p2.root);
        assert_eq!(p1.content_dir, p2.content_dir);
        assert_eq!(p1.db, p2.db);
    }

    #[cfg(unix)]
    #[test]
    fn init_storage_at_sets_unix_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let p = init_storage_at(tmp.path()).unwrap();

        let root_mode = fs::metadata(&p.root).unwrap().permissions().mode() & 0o777;
        let content_mode = fs::metadata(&p.content_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(root_mode, 0o700);
        assert_eq!(content_mode, 0o700);
    }
}
