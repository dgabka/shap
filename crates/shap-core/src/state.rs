//! Machine-written active state (`state.json`).
//!
//! Records the user's current selections so they persist across shells
//! (FR-012). Written atomically (temp file + rename). A missing file is a
//! fresh install (all-null); a corrupt file is repaired to defaults rather
//! than crashing. Read by the prompt segment, so it must stay cheap.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{Error, Result};

/// The user's current selections.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ActiveState {
    pub active_agent: Option<String>,
    pub active_model: Option<String>,
    pub active_reasoning: Option<String>,
    pub active_session_id: Option<String>,
    pub last_cwd: Option<String>,
}

impl ActiveState {
    /// Load state from `path`. A missing file ⇒ all-null (fresh install). A
    /// corrupt file is logged and treated as fresh rather than fatal.
    pub fn load(path: &Path) -> Result<ActiveState> {
        match std::fs::read_to_string(path) {
            Ok(text) => match serde_json::from_str(&text) {
                Ok(state) => Ok(state),
                Err(e) => {
                    tracing::warn!(error = %e, path = %path.display(), "corrupt state.json; treating as fresh");
                    Ok(ActiveState::default())
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ActiveState::default()),
            Err(e) => Err(Error::io(format!("reading state {}", path.display()), e)),
        }
    }

    /// Atomically write state to `path` (temp file in the same dir + rename, so
    /// a crash mid-write never corrupts the live file).
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::io(format!("creating {}", parent.display()), e))?;
        }
        let body = serde_json::to_string_pretty(self)
            .map_err(|e| Error::AgentProtocol(format!("serializing state: {e}")))?;

        let tmp = path.with_file_name(format!(".state.{}.tmp", std::process::id()));
        std::fs::write(&tmp, body.as_bytes())
            .map_err(|e| Error::io(format!("writing {}", tmp.display()), e))?;
        std::fs::rename(&tmp, path).map_err(|e| {
            let _ = std::fs::remove_file(&tmp);
            Error::io(format!("replacing {}", path.display()), e)
        })?;
        Ok(())
    }

    /// Cross-check against config: drop an `active_agent`/`active_model` that no
    /// longer exists so a stale selection is treated as unset (repaired on the
    /// next selection). Returns `true` if anything changed.
    pub fn reconcile(&mut self, config: &Config) -> bool {
        let mut changed = false;
        if let Some(agent) = &self.active_agent {
            match config.agent(agent) {
                Some(a) => {
                    if let Some(model) = &self.active_model {
                        if !a.models.contains(model) {
                            self.active_model = None;
                            changed = true;
                        }
                    }
                }
                None => {
                    self.active_agent = None;
                    self.active_model = None;
                    changed = true;
                }
            }
        } else if self.active_model.is_some() {
            // model without an agent is meaningless
            self.active_model = None;
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn missing_file_is_all_null() {
        let dir = tempdir().unwrap();
        let s = ActiveState::load(&dir.path().join("state.json")).unwrap();
        assert_eq!(s, ActiveState::default());
    }

    #[test]
    fn round_trip_atomic_write() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/state.json");
        let state = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("gpt-5".into()),
            active_reasoning: Some("high".into()),
            active_session_id: Some("2026-05-30T12-33-10Z-codex".into()),
            last_cwd: Some("/tmp/p".into()),
        };
        state.save(&path).unwrap();
        let loaded = ActiveState::load(&path).unwrap();
        assert_eq!(loaded, state);
        // No leftover temp files in the directory.
        let leftovers: Vec<_> = std::fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftovers.is_empty(), "temp file left behind");
    }

    #[test]
    fn corrupt_file_treated_as_fresh() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("state.json");
        std::fs::write(&path, b"{ not json").unwrap();
        let s = ActiveState::load(&path).unwrap();
        assert_eq!(s, ActiveState::default());
    }

    fn config_with(models: &[&str]) -> Config {
        let toml = format!(
            "default_agent = \"codex\"\n[agents.codex]\ncommand = \"c\"\nmodels = [{}]\ndefault_model = \"{}\"\n",
            models
                .iter()
                .map(|m| format!("\"{m}\""))
                .collect::<Vec<_>>()
                .join(", "),
            models[0]
        );
        toml::from_str(&toml).unwrap()
    }

    #[test]
    fn reconcile_drops_unknown_agent() {
        let cfg = config_with(&["gpt-5"]);
        let mut s = ActiveState {
            active_agent: Some("ghost".into()),
            active_model: Some("gpt-5".into()),
            ..Default::default()
        };
        assert!(s.reconcile(&cfg));
        assert_eq!(s.active_agent, None);
        assert_eq!(s.active_model, None);
    }

    #[test]
    fn reconcile_drops_invalid_model() {
        let cfg = config_with(&["gpt-5"]);
        let mut s = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("old-model".into()),
            ..Default::default()
        };
        assert!(s.reconcile(&cfg));
        assert_eq!(s.active_agent.as_deref(), Some("codex"));
        assert_eq!(s.active_model, None);
    }

    #[test]
    fn reconcile_keeps_valid_selection() {
        let cfg = config_with(&["gpt-5", "gpt-5-thinking"]);
        let mut s = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("gpt-5-thinking".into()),
            ..Default::default()
        };
        assert!(!s.reconcile(&cfg));
    }
}
