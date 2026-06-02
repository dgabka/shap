//! Prompt-segment string builder.
//!
//! Reads the active selections from [`ActiveState`] and renders a compact
//! segment for the shell prompt. Deliberately cheap: it only formats already-
//! loaded state (no config parse, no agent launch), so the prompt stays fast
//! (NFR-2 / Constitution V).

use shap_core::state::ActiveState;

/// Build the prompt segment, e.g. `[shap codex·gpt-5·high]`. Returns an empty
/// string when nothing is selected, so the shell can render nothing.
pub fn segment(state: &ActiveState) -> String {
    let mut parts: Vec<&str> = Vec::with_capacity(3);
    if let Some(agent) = &state.active_agent {
        parts.push(agent);
    }
    if let Some(model) = &state.active_model {
        parts.push(model);
    }
    if let Some(reasoning) = &state.active_reasoning {
        parts.push(reasoning);
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("[shap {}]", parts.join("·"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_is_blank() {
        assert_eq!(segment(&ActiveState::default()), "");
    }

    #[test]
    fn agent_only() {
        let state = ActiveState {
            active_agent: Some("codex".into()),
            ..Default::default()
        };
        assert_eq!(segment(&state), "[shap codex]");
    }

    #[test]
    fn full_selection() {
        let state = ActiveState {
            active_agent: Some("codex".into()),
            active_model: Some("gpt-5".into()),
            active_reasoning: Some("high".into()),
            ..Default::default()
        };
        assert_eq!(segment(&state), "[shap codex·gpt-5·high]");
    }
}
