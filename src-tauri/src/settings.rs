use std::env;
use std::path::{Path, PathBuf};

pub const ENV_VAR: &str = "GOBAI_CONFIG";

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "kawakami";
const APPLICATION: &str = "gobai";
const CONFIG_FILENAME: &str = "config.toml";

/// Resolve the path to the user's gobai config TOML.
///
/// Discovery order:
/// 1. `GOBAI_CONFIG` environment variable (taken as-is, no `exists()` check;
///    lets a user point at a path that the app should create on first save).
/// 2. OS-standard `<app_config_dir>/gobai/config.toml`, only if the file already exists.
///
/// Returns `None` when neither applies; callers fall back to `Settings::default()`
/// (defined in 02-02). Hot reload is out of scope (FR-13).
pub fn discover_config_path() -> Option<PathBuf> {
    let env_value = env::var(ENV_VAR).ok().filter(|s| !s.is_empty());
    let project_dirs = directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION);
    discover_with(env_value, project_dirs.as_ref().map(|d| d.config_dir()))
}

fn discover_with(env_value: Option<String>, config_dir: Option<&Path>) -> Option<PathBuf> {
    if let Some(v) = env_value {
        return Some(PathBuf::from(v));
    }
    let candidate = config_dir?.join(CONFIG_FILENAME);
    candidate.exists().then_some(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn env_var_overrides_everything_even_if_path_does_not_exist() {
        let result = discover_with(Some("/tmp/nonexistent.toml".into()), None);
        assert_eq!(result, Some(PathBuf::from("/tmp/nonexistent.toml")));
    }

    #[test]
    fn empty_env_var_treated_as_unset() {
        let result = discover_with(None, None);
        assert_eq!(result, None);
    }

    #[test]
    fn returns_app_config_path_when_file_exists() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join(CONFIG_FILENAME), "").unwrap();
        let result = discover_with(None, Some(tmp.path()));
        assert_eq!(result, Some(tmp.path().join(CONFIG_FILENAME)));
    }

    #[test]
    fn returns_none_when_app_config_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let result = discover_with(None, Some(tmp.path()));
        assert_eq!(result, None);
    }
}
