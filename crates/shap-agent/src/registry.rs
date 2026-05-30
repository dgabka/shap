//! Agent registry: resolve a configured agent + active selection into a
//! launchable [`SessionOptions`].
//!
//! Pure resolution (no I/O) plus a separate PATH-availability check, so the
//! send path can resolve cheaply and surface a missing-command error with a
//! clear remediation.

use std::path::PathBuf;

use shap_core::agent::SessionOptions;
use shap_core::config::Config;
use shap_core::error::{Error, Result};
use shap_core::state::ActiveState;

/// Resolves agents against a borrowed [`Config`].
pub struct Registry<'a> {
    config: &'a Config,
}

impl<'a> Registry<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Resolve the active agent + model + reasoning into launch options.
    ///
    /// The active agent falls back to `default_agent`; the active model falls
    /// back to the agent's `default_model` when unset or invalid for the agent.
    pub fn resolve(&self, state: &ActiveState, cwd: PathBuf) -> Result<SessionOptions> {
        let name = state
            .active_agent
            .clone()
            .unwrap_or_else(|| self.config.default_agent.clone());

        let agent = self
            .config
            .agent(&name)
            .ok_or_else(|| Error::UnknownAgent {
                name: name.clone(),
                configured: self.config.agent_names().join(", "),
            })?;

        let model = match &state.active_model {
            Some(m) if agent.models.contains(m) => m.clone(),
            _ => agent.default_model.clone(),
        };

        let parts = shell_words::split(&agent.command)
            .map_err(|e| Error::AgentProtocol(format!("invalid command for agent {name}: {e}")))?;
        let mut parts = parts.into_iter();
        let command = parts.next().ok_or_else(|| Error::AgentCommandMissing {
            command: agent.command.clone(),
        })?;
        let args: Vec<String> = parts.collect();

        Ok(SessionOptions {
            agent: name,
            command,
            args,
            model,
            reasoning: state.active_reasoning.clone(),
            cwd,
            extra: agent.passthrough(),
        })
    }

    /// Check that the resolved agent command exists on PATH.
    pub fn ensure_available(opts: &SessionOptions) -> Result<()> {
        which::which(&opts.command).map_err(|_| Error::AgentCommandMissing {
            command: opts.command.clone(),
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> Config {
        toml::from_str(
            r#"
default_agent = "codex"
[agents.codex]
command = "codex-acp --acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"
[agents.claude]
command = "claude-agent-acp"
models = ["sonnet"]
default_model = "sonnet"
"#,
        )
        .unwrap()
    }

    #[test]
    fn falls_back_to_default_agent_and_model() {
        let cfg = config();
        let reg = Registry::new(&cfg);
        let opts = reg
            .resolve(&ActiveState::default(), PathBuf::from("/tmp"))
            .unwrap();
        assert_eq!(opts.agent, "codex");
        assert_eq!(opts.command, "codex-acp");
        assert_eq!(opts.args, vec!["--acp".to_string()]);
        assert_eq!(opts.model, "gpt-5-thinking");
    }

    #[test]
    fn honours_active_selection() {
        let cfg = config();
        let reg = Registry::new(&cfg);
        let state = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("gpt-5".into()),
            active_reasoning: Some("high".into()),
            ..Default::default()
        };
        let opts = reg.resolve(&state, PathBuf::from("/tmp")).unwrap();
        assert_eq!(opts.model, "gpt-5");
        assert_eq!(opts.reasoning.as_deref(), Some("high"));
    }

    #[test]
    fn invalid_active_model_falls_back_to_default() {
        let cfg = config();
        let reg = Registry::new(&cfg);
        let state = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("stale".into()),
            ..Default::default()
        };
        let opts = reg.resolve(&state, PathBuf::from("/tmp")).unwrap();
        assert_eq!(opts.model, "gpt-5-thinking");
    }

    #[test]
    fn unknown_active_agent_errors() {
        let cfg = config();
        let reg = Registry::new(&cfg);
        let state = ActiveState {
            active_agent: Some("ghost".into()),
            ..Default::default()
        };
        let err = reg.resolve(&state, PathBuf::from("/tmp")).unwrap_err();
        assert!(matches!(err, Error::UnknownAgent { .. }), "{err:?}");
    }
}
