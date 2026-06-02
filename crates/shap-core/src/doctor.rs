//! `:doctor` self-check.
//!
//! Validates the installation and surfaces actionable problems (research D11).
//! Probing PATH and the shell integration goes through a [`Probe`] trait so the
//! report is deterministic in tests; the binary uses [`RealProbe`].

use std::path::Path;

use crate::config::{Config, Picker};
use crate::error::Error;
use crate::state::ActiveState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Check {
    pub name: String,
    pub level: Level,
    pub detail: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Report {
    pub checks: Vec<Check>,
}

impl Report {
    /// Whether all critical checks pass (no `Fail`).
    pub fn ok(&self) -> bool {
        self.checks.iter().all(|c| c.level != Level::Fail)
    }
}

impl std::fmt::Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for c in &self.checks {
            let tag = match c.level {
                Level::Ok => "ok  ",
                Level::Warn => "warn",
                Level::Fail => "FAIL",
            };
            writeln!(f, "[{tag}] {}: {}", c.name, c.detail)?;
        }
        Ok(())
    }
}

/// Environment probing, injected so the report is testable.
pub trait Probe {
    /// Whether `command` resolves on PATH.
    fn command_on_path(&self, command: &str) -> bool;
    /// The shell integration marker (`SHAP_SHELL_INTEGRATION`), if set.
    fn shell_integration(&self) -> Option<String>;
}

/// Real probing via `which` and the process environment.
pub struct RealProbe;

impl Probe for RealProbe {
    fn command_on_path(&self, command: &str) -> bool {
        which::which(command).is_ok()
    }

    fn shell_integration(&self) -> Option<String> {
        std::env::var("SHAP_SHELL_INTEGRATION")
            .ok()
            .filter(|s| !s.is_empty())
    }
}

fn check(name: impl Into<String>, level: Level, detail: impl Into<String>) -> Check {
    Check {
        name: name.into(),
        level,
        detail: detail.into(),
    }
}

/// Run all checks. `config` is the caller's load result so a config error is
/// reported as a failing check rather than aborting the whole report.
pub fn run(
    config: Result<&Config, &Error>,
    state: &ActiveState,
    sessions_dir: &Path,
    probe: &dyn Probe,
) -> Report {
    let mut checks = Vec::new();

    let config = match config {
        Ok(c) => {
            checks.push(check("config", Level::Ok, "loaded and valid"));
            Some(c)
        }
        Err(e) => {
            checks.push(check("config", Level::Fail, e.to_string()));
            None
        }
    };

    if probe.command_on_path("git") {
        checks.push(check("git", Level::Ok, "available"));
    } else {
        checks.push(check("git", Level::Warn, "not found; `:commit` needs git"));
    }

    if let Some(config) = config {
        let picker_bin = match config.ui.picker {
            Picker::Fzf => Some("fzf"),
            Picker::Skim => Some("sk"),
            Picker::Builtin => None,
        };
        if let Some(bin) = picker_bin {
            if probe.command_on_path(bin) {
                checks.push(check("picker", Level::Ok, format!("`{bin}` available")));
            } else {
                checks.push(check(
                    "picker",
                    Level::Warn,
                    format!("`{bin}` not found; falling back to the built-in picker"),
                ));
            }
        }

        for (name, agent) in &config.agents {
            let program = agent.command.split_whitespace().next().unwrap_or("");
            if probe.command_on_path(program) {
                checks.push(check(
                    format!("agent:{name}"),
                    Level::Ok,
                    format!("`{program}` on PATH"),
                ));
            } else {
                checks.push(check(
                    format!("agent:{name}"),
                    Level::Fail,
                    format!("`{program}` not found on PATH"),
                ));
            }
        }

        if let Some(active) = &state.active_agent {
            match config.agent(active) {
                None => checks.push(check(
                    "selection",
                    Level::Warn,
                    format!("active agent `{active}` is not configured (will reset)"),
                )),
                Some(agent) => {
                    if let Some(model) = &state.active_model {
                        if !agent.models.contains(model) {
                            checks.push(check(
                                "selection",
                                Level::Warn,
                                format!(
                                    "active model `{model}` is invalid for `{active}` (will reset)"
                                ),
                            ));
                        }
                    }
                }
            }
        }
    }

    if writable(sessions_dir) {
        checks.push(check("sessions", Level::Ok, "directory is writable"));
    } else {
        checks.push(check(
            "sessions",
            Level::Fail,
            "cannot write to the session directory",
        ));
    }

    match probe.shell_integration() {
        Some(kind) => checks.push(check(
            "shell",
            Level::Ok,
            format!("integration active ({kind})"),
        )),
        None => checks.push(check(
            "shell",
            Level::Warn,
            "integration not detected; source shell/zsh/shap.zsh",
        )),
    }

    Report { checks }
}

fn writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(".shap-doctor-write-test");
    let ok = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::tempdir;

    struct FakeProbe {
        available: HashSet<String>,
        shell: Option<String>,
    }

    impl Probe for FakeProbe {
        fn command_on_path(&self, command: &str) -> bool {
            self.available.contains(command)
        }
        fn shell_integration(&self) -> Option<String> {
            self.shell.clone()
        }
    }

    fn config() -> Config {
        toml::from_str(
            r#"
default_agent = "codex"
[agents.claude]
command = "claude-agent-acp"
models = ["sonnet"]
default_model = "sonnet"
[agents.codex]
command = "codex-acp --acp"
models = ["gpt-5"]
default_model = "gpt-5"
"#,
        )
        .unwrap()
    }

    #[test]
    fn healthy_report_snapshot() {
        let dir = tempdir().unwrap();
        let cfg = config();
        let probe = FakeProbe {
            available: ["git", "fzf", "codex-acp", "claude-agent-acp"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            shell: Some("zsh".to_string()),
        };
        let report = run(Ok(&cfg), &ActiveState::default(), dir.path(), &probe);
        assert!(report.ok());
        insta::assert_snapshot!("doctor_healthy", report.to_string());
    }

    #[test]
    fn missing_agent_report_snapshot() {
        let dir = tempdir().unwrap();
        let cfg = config();
        let probe = FakeProbe {
            // codex-acp is absent → its check fails; fzf absent → picker warn.
            available: ["git", "claude-agent-acp"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            shell: None,
        };
        let report = run(Ok(&cfg), &ActiveState::default(), dir.path(), &probe);
        assert!(
            !report.ok(),
            "a missing agent command is a critical failure"
        );
        insta::assert_snapshot!("doctor_missing_agent", report.to_string());
    }
}
