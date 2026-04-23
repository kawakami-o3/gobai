use serde::Deserialize;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmLevel {
    Strict,
    #[default]
    Normal,
    Autonomous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinorPolicy {
    #[default]
    RecordAndContinue,
    BlockUntilResolved,
    PromptUser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeCleanup {
    #[default]
    Keep,
    Remove,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LoopSettings {
    pub design_max: u32,
    pub impl_max: u32,
    pub cli_failure_max: u32,
    pub extension_step: u32,
}

impl Default for LoopSettings {
    fn default() -> Self {
        Self {
            design_max: 3,
            impl_max: 3,
            cli_failure_max: 2,
            extension_step: 2,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CostSettings {
    pub max_api_calls_per_task: u32,
    pub max_tokens_per_task: u64,
    pub warn_at_ratio: f32,
}

impl Default for CostSettings {
    fn default() -> Self {
        Self {
            max_api_calls_per_task: 60,
            max_tokens_per_task: 1_500_000,
            warn_at_ratio: 0.8_f32,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WorktreeSettings {
    pub cleanup_on_done: WorktreeCleanup,
    pub cleanup_on_abort: WorktreeCleanup,
    pub base_ref: String,
    pub prune_orphans_on_startup: bool,
}

impl Default for WorktreeSettings {
    fn default() -> Self {
        Self {
            cleanup_on_done: WorktreeCleanup::Keep,
            cleanup_on_abort: WorktreeCleanup::Keep,
            base_ref: "main".to_string(),
            prune_orphans_on_startup: true,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct AgentSettings {
    pub codex_model: Option<String>,
    pub claude_model: Option<String>,
    pub timeout_secs: Option<u32>,
    pub retries: Option<u32>,
    pub include_extra_summary_paths: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct LogSettings {
    pub redaction_patterns: Vec<String>,
    pub task_size_limit_mb: u32,
    pub global_soft_limit_gb: u32,
}

impl Default for LogSettings {
    fn default() -> Self {
        Self {
            redaction_patterns: vec![
                "AWS_*".to_string(),
                "GCP_*".to_string(),
                "GITHUB_TOKEN".to_string(),
                "Bearer .*".to_string(),
            ],
            task_size_limit_mb: 100,
            global_soft_limit_gb: 5,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub notifications: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub confirm_level: ConfirmLevel,
    pub minor_policy: MinorPolicy,
    #[serde(rename = "loop")]
    pub loop_: LoopSettings,
    pub cost: CostSettings,
    pub worktree: WorktreeSettings,
    pub agent: AgentSettings,
    pub log: LogSettings,
    pub ui: UiSettings,
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

    #[test]
    fn default_root_enums_match_spec() {
        let s = Settings::default();
        assert_eq!(s.confirm_level, ConfirmLevel::Normal);
        assert_eq!(s.minor_policy, MinorPolicy::RecordAndContinue);
    }

    #[test]
    fn default_loop_caps_match_adr_003() {
        let l = Settings::default().loop_;
        assert_eq!(l.design_max, 3);
        assert_eq!(l.impl_max, 3);
        assert_eq!(l.cli_failure_max, 2);
        assert_eq!(l.extension_step, 2);
    }

    #[test]
    fn default_cost_caps_match_adr_003() {
        let c = Settings::default().cost;
        assert_eq!(c.max_api_calls_per_task, 60);
        assert_eq!(c.max_tokens_per_task, 1_500_000);
        assert!((c.warn_at_ratio - 0.8_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn default_worktree_matches_adr_002() {
        let w = Settings::default().worktree;
        assert_eq!(w.cleanup_on_done, WorktreeCleanup::Keep);
        assert_eq!(w.cleanup_on_abort, WorktreeCleanup::Keep);
        assert_eq!(w.base_ref, "main");
        assert!(w.prune_orphans_on_startup);
    }

    #[test]
    fn default_agent_is_all_none_except_paths() {
        let a = Settings::default().agent;
        assert_eq!(a.codex_model, None);
        assert_eq!(a.claude_model, None);
        assert_eq!(a.timeout_secs, None);
        assert_eq!(a.retries, None);
        assert!(a.include_extra_summary_paths.is_empty());
    }

    #[test]
    fn default_log_redaction_includes_aws_gcp_github_bearer() {
        let patterns = Settings::default().log.redaction_patterns;
        assert!(patterns.iter().any(|p| p == "AWS_*"));
        assert!(patterns.iter().any(|p| p == "GCP_*"));
        assert!(patterns.iter().any(|p| p == "GITHUB_TOKEN"));
        assert!(patterns.iter().any(|p| p == "Bearer .*"));
    }

    #[test]
    fn default_log_size_limits_match_nfr_8() {
        let l = Settings::default().log;
        assert_eq!(l.task_size_limit_mb, 100);
        assert_eq!(l.global_soft_limit_gb, 5);
    }

    #[test]
    fn default_ui_is_all_none() {
        let u = Settings::default().ui;
        assert_eq!(u.theme, None);
        assert_eq!(u.language, None);
        assert_eq!(u.notifications, None);
    }
}
