//! User configuration (`config.toml`).
//!
//! Loaded read-only on the hot path; the only writes are user-initiated, via
//! the first-run wizard and `shap config edit` ([`Config::write`], atomic and
//! validate-first). Validation runs on load and produces actionable
//! diagnostics (see [`crate::error`]). Unknown keys under `[agents.<name>]` are
//! preserved as opaque passthrough (FR-022) and survive a write round-trip.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::paths::{EnvVars, Paths, expand_path};

/// Top-level user configuration.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Config {
    /// Agent used when no active agent is set. MUST be a key in `agents`.
    pub default_agent: String,
    /// Configured agents, keyed by name.
    pub agents: BTreeMap<String, Agent>,
    #[serde(default)]
    pub ui: UiOptions,
    #[serde(default)]
    pub history: HistoryOptions,
    #[serde(default)]
    pub files: FileOptions,
}

/// A configured ACP agent.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct Agent {
    /// Launch command for the external ACP process (validated by `:doctor`).
    pub command: String,
    /// Valid models for this agent. Non-empty.
    pub models: Vec<String>,
    /// Default model. MUST be a member of `models`.
    pub default_model: String,
    /// Arbitrary agent-specific config, forwarded verbatim (FR-022).
    #[serde(flatten)]
    #[schemars(skip)]
    pub extra: toml::Table,
}

impl Agent {
    /// Passthrough config flattened to string scalars for transport.
    pub fn passthrough(&self) -> BTreeMap<String, String> {
        self.extra
            .iter()
            .map(|(k, v)| {
                let s = match v {
                    toml::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                (k.clone(), s)
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Picker {
    Fzf,
    Skim,
    Builtin,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct UiOptions {
    /// Streamed vs. loader-then-final output (FR-003).
    pub stream: bool,
    /// Picker preference; runtime falls back if unavailable.
    pub picker: Picker,
    /// Toggles the optional prompt segment (FR-008).
    pub show_prompt_segment: bool,
}

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            stream: true,
            picker: Picker::Fzf,
            show_prompt_segment: true,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct HistoryOptions {
    /// Session directory (`~`/`$XDG_*` expanded). `None` ⇒ data-dir default.
    pub dir: Option<String>,
    /// Reserved for future auto-capture; MVP capture is explicit.
    pub capture_last_output: bool,
    /// Upper bound on captured output sent to an agent. MUST be > 0.
    pub max_output_bytes: u64,
}

impl Default for HistoryOptions {
    fn default() -> Self {
        Self {
            dir: None,
            capture_last_output: false,
            max_output_bytes: 200_000,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct FileOptions {
    /// Max size of an `@file` inclusion. MUST be > 0.
    pub max_file_bytes: u64,
    /// Skip gitignored files during `@file` resolution.
    pub respect_gitignore: bool,
}

impl Default for FileOptions {
    fn default() -> Self {
        Self {
            max_file_bytes: 200_000,
            respect_gitignore: true,
        }
    }
}

impl Config {
    /// Load and validate config from `path`. A missing file yields
    /// [`Error::ConfigNotFound`] (setup instructions, not a crash).
    pub fn load(path: &Path) -> Result<Config> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::ConfigNotFound {
                    path: path.to_path_buf(),
                });
            }
            Err(e) => {
                return Err(Error::io(format!("reading config {}", path.display()), e));
            }
        };
        let config: Config = toml::from_str(&text).map_err(|source| Error::ConfigParse {
            path: path.to_path_buf(),
            source,
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Validate, then atomically write the config to `path` (temp file in the
    /// same dir + rename, so a crash never leaves a partial config). Never
    /// writes an invalid config (FR-004/FR-007). Mirrors [`crate::state`]'s
    /// atomic-write pattern.
    pub fn write(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let body = toml::to_string_pretty(self)
            .map_err(|e| Error::AgentProtocol(format!("serializing config: {e}")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| Error::ConfigWriteFailed {
                path: path.to_path_buf(),
                source: e,
            })?;
        }
        let tmp = path.with_file_name(format!(".config.{}.tmp", std::process::id()));
        std::fs::write(&tmp, body.as_bytes()).map_err(|e| Error::ConfigWriteFailed {
            path: tmp.clone(),
            source: e,
        })?;
        std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            Error::ConfigWriteFailed {
                path: path.to_path_buf(),
                source: e,
            }
        })?;
        Ok(())
    }

    /// Validate cross-field invariants (data-model § Validation summary).
    pub fn validate(&self) -> Result<()> {
        if self.agents.is_empty() {
            return Err(Error::NoAgentConfigured);
        }
        if !self.agents.contains_key(&self.default_agent) {
            return Err(Error::UnknownDefaultAgent {
                agent: self.default_agent.clone(),
                configured: self.agent_names().join(", "),
            });
        }
        for (name, agent) in &self.agents {
            if agent.models.is_empty() {
                return Err(Error::AgentEmptyModels {
                    agent: name.clone(),
                });
            }
            if !agent.models.contains(&agent.default_model) {
                return Err(Error::DefaultModelNotInModels {
                    agent: name.clone(),
                    default_model: agent.default_model.clone(),
                    models: agent.models.join(", "),
                });
            }
        }
        if self.history.max_output_bytes == 0 {
            return Err(Error::NonPositiveByteLimit {
                field: "history.max_output_bytes",
            });
        }
        if self.files.max_file_bytes == 0 {
            return Err(Error::NonPositiveByteLimit {
                field: "files.max_file_bytes",
            });
        }
        Ok(())
    }

    /// Configured agent names, sorted (BTreeMap order).
    pub fn agent_names(&self) -> Vec<String> {
        self.agents.keys().cloned().collect()
    }

    pub fn agent(&self, name: &str) -> Option<&Agent> {
        self.agents.get(name)
    }

    /// Resolve the session directory, expanding `~`/`$XDG_*` or falling back to
    /// the data-dir default.
    pub fn sessions_dir(&self, paths: &Paths, env: &EnvVars) -> PathBuf {
        match &self.history.dir {
            Some(raw) => expand_path(raw, env),
            None => paths.default_sessions_dir(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_toml() -> &'static str {
        r#"
default_agent = "codex"

[agents.codex]
command = "codex-acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"
api_key_env = "OPENAI_API_KEY"
"#
    }

    fn parse(s: &str) -> Config {
        toml::from_str(s).expect("parse")
    }

    #[test]
    fn defaults_apply_when_sections_absent() {
        let c = parse(valid_toml());
        c.validate().expect("valid");
        assert!(c.ui.stream);
        assert_eq!(c.ui.picker, Picker::Fzf);
        assert!(c.ui.show_prompt_segment);
        assert_eq!(c.history.max_output_bytes, 200_000);
        assert_eq!(c.files.max_file_bytes, 200_000);
        assert!(c.files.respect_gitignore);
    }

    #[test]
    fn passthrough_is_preserved() {
        let c = parse(valid_toml());
        let extra = c.agents["codex"].passthrough();
        assert_eq!(
            extra.get("api_key_env").map(String::as_str),
            Some("OPENAI_API_KEY")
        );
    }

    #[test]
    fn default_agent_must_be_configured() {
        let s = r#"
default_agent = "missing"
[agents.codex]
command = "c"
models = ["m"]
default_model = "m"
"#;
        let err = parse(s).validate().unwrap_err();
        assert!(matches!(err, Error::UnknownDefaultAgent { .. }), "{err:?}");
    }

    #[test]
    fn default_model_must_be_in_models() {
        let s = r#"
default_agent = "codex"
[agents.codex]
command = "c"
models = ["a", "b"]
default_model = "c"
"#;
        let err = parse(s).validate().unwrap_err();
        assert!(
            matches!(err, Error::DefaultModelNotInModels { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn empty_models_rejected() {
        let s = r#"
default_agent = "codex"
[agents.codex]
command = "c"
models = []
default_model = "m"
"#;
        let err = parse(s).validate().unwrap_err();
        assert!(matches!(err, Error::AgentEmptyModels { .. }), "{err:?}");
    }

    #[test]
    fn zero_byte_limits_rejected() {
        let s = r#"
default_agent = "codex"
[agents.codex]
command = "c"
models = ["m"]
default_model = "m"
[history]
max_output_bytes = 0
"#;
        let err = parse(s).validate().unwrap_err();
        assert!(
            matches!(
                err,
                Error::NonPositiveByteLimit {
                    field: "history.max_output_bytes"
                }
            ),
            "{err:?}"
        );
    }

    #[test]
    fn missing_file_is_config_not_found() {
        let err = Config::load(Path::new("/nonexistent/shap/config.toml")).unwrap_err();
        assert!(matches!(err, Error::ConfigNotFound { .. }), "{err:?}");
    }

    #[test]
    fn write_round_trip_preserves_passthrough_and_validates() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/config.toml");
        let c = parse(valid_toml());
        c.write(&path).expect("write");
        let reloaded = Config::load(&path).expect("reload");
        // FR-008: opaque passthrough survives the write round-trip.
        assert_eq!(
            reloaded.agents["codex"].passthrough().get("api_key_env"),
            Some(&"OPENAI_API_KEY".to_string())
        );
        assert_eq!(reloaded.default_agent, "codex");
    }

    #[test]
    fn write_rejects_invalid_config_and_creates_no_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut c = parse(valid_toml());
        c.default_agent = "missing".to_string();
        let err = c.write(&path).unwrap_err();
        assert!(matches!(err, Error::UnknownDefaultAgent { .. }), "{err:?}");
        assert!(
            !path.exists(),
            "no file must be written on validation failure"
        );
    }

    #[test]
    fn invalid_picker_value_is_rejected_at_parse() {
        let s = r#"
default_agent = "codex"
[agents.codex]
command = "c"
models = ["m"]
default_model = "m"
[ui]
picker = "rofi"
"#;
        assert!(toml::from_str::<Config>(s).is_err());
    }
}
