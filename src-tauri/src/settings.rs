use figment::providers::{Env, Format, Toml};
use figment::Figment;
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmLevel {
    Strict,
    #[default]
    Normal,
    Autonomous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinorPolicy {
    #[default]
    RecordAndContinue,
    BlockUntilResolved,
    PromptUser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorktreeCleanup {
    #[default]
    Keep,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentSettings {
    pub codex_model: Option<String>,
    pub claude_model: Option<String>,
    pub timeout_secs: Option<u32>,
    pub retries: Option<u32>,
    pub include_extra_summary_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct UiSettings {
    pub theme: Option<String>,
    pub language: Option<String>,
    pub notifications: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// Errors surfaced by `validate`. Aggregated and returned together so callers
/// can show every problem to the user in one shot rather than fix-and-retry.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    NonPositive {
        field: &'static str,
        value: String,
    },
    OutOfRange {
        field: &'static str,
        value: String,
        min: f64,
        max: f64,
    },
    InvalidRegex {
        field: &'static str,
        index: usize,
        pattern: String,
        reason: String,
    },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonPositive { field, value } => {
                write!(f, "{field} は正の値である必要があります (現在: {value})")
            }
            Self::OutOfRange {
                field,
                value,
                min,
                max,
            } => write!(
                f,
                "{field} は {min} 以上 {max} 以下である必要があります (現在: {value})"
            ),
            Self::InvalidRegex {
                field,
                index,
                pattern,
                reason,
            } => write!(
                f,
                "{field}[{index}] の正規表現がコンパイルできません (\"{pattern}\"): {reason}"
            ),
        }
    }
}

/// Combined error type for the load → validate pipeline.
#[derive(Debug)]
pub enum SettingsError {
    Load(figment::Error),
    Validation(Vec<ValidationError>),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Load(e) => write!(f, "設定ファイルの読み込みに失敗しました: {e}"),
            Self::Validation(errs) => {
                writeln!(f, "設定値が不正です:")?;
                for e in errs {
                    writeln!(f, "  - {e}")?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for SettingsError {}

impl From<figment::Error> for SettingsError {
    fn from(e: figment::Error) -> Self {
        Self::Load(e)
    }
}

/// Validate semantic constraints not expressible by serde/figment alone.
///
/// Errors are collected (not short-circuited) so all problems can be reported
/// to the user at once. Enum-typed fields (e.g. `confirm_level`) are already
/// constrained at deserialization time and need no checks here.
pub fn validate(s: &Settings) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    let positives_u32: &[(&'static str, u32)] = &[
        ("loop.design_max", s.loop_.design_max),
        ("loop.impl_max", s.loop_.impl_max),
        ("loop.cli_failure_max", s.loop_.cli_failure_max),
        ("loop.extension_step", s.loop_.extension_step),
        ("cost.max_api_calls_per_task", s.cost.max_api_calls_per_task),
        ("log.task_size_limit_mb", s.log.task_size_limit_mb),
        ("log.global_soft_limit_gb", s.log.global_soft_limit_gb),
    ];
    for (field, value) in positives_u32 {
        if *value == 0 {
            errors.push(ValidationError::NonPositive {
                field,
                value: value.to_string(),
            });
        }
    }

    if s.cost.max_tokens_per_task == 0 {
        errors.push(ValidationError::NonPositive {
            field: "cost.max_tokens_per_task",
            value: s.cost.max_tokens_per_task.to_string(),
        });
    }

    if let Some(t) = s.agent.timeout_secs {
        if t == 0 {
            errors.push(ValidationError::NonPositive {
                field: "agent.timeout_secs",
                value: t.to_string(),
            });
        }
    }

    let ratio = s.cost.warn_at_ratio;
    if !(0.0..=1.0).contains(&ratio) || ratio.is_nan() {
        errors.push(ValidationError::OutOfRange {
            field: "cost.warn_at_ratio",
            value: ratio.to_string(),
            min: 0.0,
            max: 1.0,
        });
    }

    for (i, pattern) in s.log.redaction_patterns.iter().enumerate() {
        if let Err(e) = regex::Regex::new(pattern) {
            errors.push(ValidationError::InvalidRegex {
                field: "log.redaction_patterns",
                index: i,
                pattern: pattern.clone(),
                reason: e.to_string(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Load `Settings` by merging (in order): TOML file (if discovered) and env vars,
/// then validate semantic constraints.
///
/// - Missing config file: skipped, defaults are used.
/// - Env vars are read with prefix `GOBAI_SETTINGS_` (e.g. `GOBAI_SETTINGS_CONFIRM_LEVEL=strict`).
/// - Parse / type errors return `SettingsError::Load`; semantic errors return `SettingsError::Validation`.
#[allow(clippy::result_large_err)]
pub fn load_settings() -> Result<Settings, SettingsError> {
    let mut fig = Figment::new();
    if let Some(path) = discover_config_path() {
        fig = fig.merge(Toml::file(path));
    }
    let settings: Settings = fig.merge(Env::prefixed("GOBAI_SETTINGS_")).extract()?;
    validate(&settings).map_err(SettingsError::Validation)?;
    Ok(settings)
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

    #[test]
    fn load_returns_defaults_when_no_config_or_env() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            let s = load_settings().unwrap();
            assert_eq!(s.confirm_level, ConfirmLevel::Normal);
            assert_eq!(s.minor_policy, MinorPolicy::RecordAndContinue);
            Ok(())
        });
    }

    #[test]
    fn load_reads_toml_file_via_env_var() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            jail.create_file("config.toml", r#"confirm_level = "strict""#)?;
            let path = jail.directory().join("config.toml");
            jail.set_env(ENV_VAR, path.to_str().unwrap());
            let s = load_settings().unwrap();
            assert_eq!(s.confirm_level, ConfirmLevel::Strict);
            Ok(())
        });
    }

    #[test]
    fn env_var_overrides_toml() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            jail.create_file("config.toml", r#"confirm_level = "strict""#)?;
            let path = jail.directory().join("config.toml");
            jail.set_env(ENV_VAR, path.to_str().unwrap());
            jail.set_env("GOBAI_SETTINGS_CONFIRM_LEVEL", "autonomous");
            let s = load_settings().unwrap();
            assert_eq!(s.confirm_level, ConfirmLevel::Autonomous);
            Ok(())
        });
    }

    #[test]
    fn validate_accepts_defaults() {
        assert!(validate(&Settings::default()).is_ok());
    }

    #[test]
    fn validate_rejects_zero_design_max() {
        let mut s = Settings::default();
        s.loop_.design_max = 0;
        let errs = validate(&s).unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::NonPositive { field, .. } if *field == "loop.design_max"
        )));
    }

    #[test]
    fn validate_rejects_warn_at_ratio_above_one() {
        let mut s = Settings::default();
        s.cost.warn_at_ratio = 1.5;
        let errs = validate(&s).unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::OutOfRange { field, .. } if *field == "cost.warn_at_ratio"
        )));
    }

    #[test]
    fn validate_rejects_warn_at_ratio_negative() {
        let mut s = Settings::default();
        s.cost.warn_at_ratio = -0.1;
        let errs = validate(&s).unwrap_err();
        assert!(errs.iter().any(|e| matches!(
            e,
            ValidationError::OutOfRange { field, .. } if *field == "cost.warn_at_ratio"
        )));
    }

    #[test]
    fn validate_rejects_invalid_regex() {
        let mut s = Settings::default();
        s.log.redaction_patterns = vec!["[invalid".to_string()];
        let errs = validate(&s).unwrap_err();
        assert!(matches!(
            errs.first(),
            Some(ValidationError::InvalidRegex { field, index: 0, .. })
                if *field == "log.redaction_patterns"
        ));
    }

    #[test]
    fn validate_collects_multiple_errors() {
        let mut s = Settings::default();
        s.loop_.design_max = 0;
        s.cost.warn_at_ratio = 2.0;
        let errs = validate(&s).unwrap_err();
        assert_eq!(errs.len(), 2);
    }

    #[test]
    fn validate_skips_unset_optional_timeout() {
        let mut s = Settings::default();
        s.agent.timeout_secs = None;
        assert!(validate(&s).is_ok());
        s.agent.timeout_secs = Some(0);
        assert!(validate(&s).is_err());
    }

    #[test]
    fn load_settings_returns_validation_error_for_bad_toml() {
        figment::Jail::expect_with(|jail| {
            jail.clear_env();
            jail.create_file(
                "config.toml",
                r#"
[loop]
design_max = 0
"#,
            )?;
            let path = jail.directory().join("config.toml");
            jail.set_env(ENV_VAR, path.to_str().unwrap());
            match load_settings() {
                Err(SettingsError::Validation(errs)) => {
                    assert!(errs.iter().any(|e| matches!(
                        e,
                        ValidationError::NonPositive { field, .. } if *field == "loop.design_max"
                    )));
                }
                other => panic!("expected Validation error, got {other:?}"),
            }
            Ok(())
        });
    }
}
