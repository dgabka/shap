//! Interactive config setup wizard (first run) and editor.
//!
//! The wizard (US1) builds a basic [`Config`] from a few prompts when none
//! exists; the editor (US2) mutates an existing config. Both produce a
//! validated `Config` that the caller persists via [`Config::write`] (atomic,
//! validate-first). Prompts use `dialoguer` (already vendored); declining or
//! cancelling returns `Ok(None)`, leaving nothing behind (FR-005/FR-009).

use std::collections::BTreeMap;

use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, Select};

use crate::config::{Agent, Config, FileOptions, HistoryOptions, Picker, UiOptions};
use crate::error::{Error, Result};

/// A built-in agent preset offered to speed up first-run setup (research D6).
struct Preset {
    name: &'static str,
    command: &'static str,
    models: &'static [&'static str],
}

const PRESETS: &[Preset] = &[
    Preset {
        name: "claude",
        command: "claude-agent-acp",
        models: &["sonnet", "opus"],
    },
    Preset {
        name: "codex",
        command: "codex-acp",
        models: &["gpt-5", "gpt-5-thinking"],
    },
];

/// Map a dialoguer interaction error (incl. user interrupt) to a domain error.
fn wiz_err(e: dialoguer::Error) -> Error {
    Error::PickerFailed {
        detail: e.to_string(),
    }
}

/// Split a free-text model entry on whitespace/commas, dropping empties.
fn parse_models(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// A single agent's answers, gathered before becoming a [`Config`]/[`Agent`].
///
/// Pure: holds no I/O state. `into_*` are unit-testable without prompting.
pub struct WizardDraft {
    pub agent_name: String,
    pub command: String,
    pub models: Vec<String>,
    pub default_model: String,
}

impl WizardDraft {
    /// Build the `(name, Agent)` pair (empty passthrough).
    pub fn into_agent(self) -> (String, Agent) {
        (
            self.agent_name,
            Agent {
                command: self.command,
                models: self.models,
                default_model: self.default_model,
                extra: toml::Table::new(),
            },
        )
    }

    /// Build a single-agent `Config` with default ui/history/files. The agent
    /// is also the `default_agent`.
    pub fn into_config(self) -> Config {
        let (name, agent) = self.into_agent();
        let mut agents = BTreeMap::new();
        agents.insert(name.clone(), agent);
        Config {
            default_agent: name,
            agents,
            ui: UiOptions::default(),
            history: HistoryOptions::default(),
            files: FileOptions::default(),
        }
    }
}

/// Prompt for non-empty models, pre-filled from `defaults` when available.
fn prompt_models(theme: &ColorfulTheme, defaults: &[String]) -> Result<Vec<String>> {
    let default_str = defaults.join(" ");
    loop {
        let mut input =
            Input::<String>::with_theme(theme).with_prompt("Models (space/comma separated)");
        if !default_str.is_empty() {
            input = input.default(default_str.clone());
        }
        let raw = input.interact_text().map_err(wiz_err)?;
        let models = parse_models(&raw);
        if !models.is_empty() {
            return Ok(models);
        }
        eprintln!("Enter at least one model.");
    }
}

/// Prompt for one agent (preset or custom) and its models/default model.
fn prompt_agent(theme: &ColorfulTheme) -> Result<WizardDraft> {
    let mut items: Vec<String> = PRESETS
        .iter()
        .map(|p| format!("{} ({})", p.name, p.command))
        .collect();
    items.push("custom".to_string());

    let idx = Select::with_theme(theme)
        .with_prompt("Choose an agent")
        .items(&items)
        .default(0)
        .interact()
        .map_err(wiz_err)?;

    let (agent_name, command, preset_models) = if idx < PRESETS.len() {
        let p = &PRESETS[idx];
        (
            p.name.to_string(),
            p.command.to_string(),
            p.models.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
    } else {
        let name: String = Input::with_theme(theme)
            .with_prompt("Agent name")
            .interact_text()
            .map_err(wiz_err)?;
        let command: String = Input::with_theme(theme)
            .with_prompt("Launch command")
            .interact_text()
            .map_err(wiz_err)?;
        (name, command, Vec::new())
    };

    let models = prompt_models(theme, &preset_models)?;
    let default_model = if models.len() == 1 {
        models[0].clone()
    } else {
        let di = Select::with_theme(theme)
            .with_prompt("Default model")
            .items(&models)
            .default(0)
            .interact()
            .map_err(wiz_err)?;
        models[di].clone()
    };

    Ok(WizardDraft {
        agent_name,
        command,
        models,
        default_model,
    })
}

/// Prompt for UI options, defaulting to [`UiOptions::default`].
fn prompt_ui(theme: &ColorfulTheme) -> Result<UiOptions> {
    let use_defaults = Confirm::with_theme(theme)
        .with_prompt("Use default UI settings (streaming on, fzf picker, prompt segment on)?")
        .default(true)
        .interact()
        .map_err(wiz_err)?;
    if use_defaults {
        return Ok(UiOptions::default());
    }
    let stream = Confirm::with_theme(theme)
        .with_prompt("Stream replies?")
        .default(true)
        .interact()
        .map_err(wiz_err)?;
    let pickers = ["fzf", "skim", "builtin"];
    let pidx = Select::with_theme(theme)
        .with_prompt("Picker")
        .items(pickers)
        .default(0)
        .interact()
        .map_err(wiz_err)?;
    let picker = match pidx {
        1 => Picker::Skim,
        2 => Picker::Builtin,
        _ => Picker::Fzf,
    };
    let show_prompt_segment = Confirm::with_theme(theme)
        .with_prompt("Show prompt segment?")
        .default(true)
        .interact()
        .map_err(wiz_err)?;
    Ok(UiOptions {
        stream,
        picker,
        show_prompt_segment,
    })
}

/// First-run wizard. Returns `Ok(Some(config))` to write, or `Ok(None)` if the
/// user declines/cancels (caller leaves no file behind).
pub fn run_wizard() -> Result<Option<Config>> {
    let theme = ColorfulTheme::default();

    let proceed = Confirm::with_theme(&theme)
        .with_prompt("No config found. Set one up now?")
        .default(true)
        .interact()
        .map_err(wiz_err)?;
    if !proceed {
        return Ok(None);
    }

    let draft = prompt_agent(&theme)?;
    let ui = prompt_ui(&theme)?;
    let mut config = draft.into_config();
    config.ui = ui;

    let preview = toml::to_string_pretty(&config)
        .map_err(|e| Error::AgentProtocol(format!("serializing config: {e}")))?;
    println!("\n{preview}");

    let write = Confirm::with_theme(&theme)
        .with_prompt("Write this config?")
        .default(true)
        .interact()
        .map_err(wiz_err)?;
    if !write {
        return Ok(None);
    }
    Ok(Some(config))
}

/// Pick one of the configured agent names, defaulting to `current` if present.
fn pick_agent(theme: &ColorfulTheme, config: &Config, current: Option<&str>) -> Result<String> {
    let names: Vec<String> = config.agents.keys().cloned().collect();
    let default = current
        .and_then(|c| names.iter().position(|n| n == c))
        .unwrap_or(0);
    let i = Select::with_theme(theme)
        .with_prompt("Agent")
        .items(&names)
        .default(default)
        .interact()
        .map_err(wiz_err)?;
    Ok(names[i].clone())
}

/// Interactive editor over an existing `Config`. Returns `Ok(Some(updated))`
/// when the user saves a change, `Ok(None)` on cancel or no change.
pub fn run_editor(mut config: Config) -> Result<Option<Config>> {
    let theme = ColorfulTheme::default();
    let mut dirty = false;

    let actions = [
        "Change default agent",
        "Add a model to an agent",
        "Set an agent's default model",
        "Add an agent",
        "Change picker",
        "Toggle streaming",
        "Toggle prompt segment",
        "Save & exit",
        "Cancel",
    ];

    loop {
        let idx = Select::with_theme(&theme)
            .with_prompt("Edit config")
            .items(actions)
            .default(0)
            .interact()
            .map_err(wiz_err)?;

        match idx {
            0 => {
                let name = pick_agent(&theme, &config, Some(&config.default_agent))?;
                config.default_agent = name;
                dirty = true;
            }
            1 => {
                let name = pick_agent(&theme, &config, None)?;
                let model: String = Input::with_theme(&theme)
                    .with_prompt("New model")
                    .interact_text()
                    .map_err(wiz_err)?;
                if let Some(a) = config.agents.get_mut(&name) {
                    if !a.models.contains(&model) {
                        a.models.push(model);
                        dirty = true;
                    }
                }
            }
            2 => {
                let name = pick_agent(&theme, &config, None)?;
                if let Some(a) = config.agents.get_mut(&name) {
                    let di = Select::with_theme(&theme)
                        .with_prompt("Default model")
                        .items(&a.models)
                        .default(0)
                        .interact()
                        .map_err(wiz_err)?;
                    a.default_model = a.models[di].clone();
                    dirty = true;
                }
            }
            3 => {
                let draft = prompt_agent(&theme)?;
                let (name, agent) = draft.into_agent();
                config.agents.insert(name, agent);
                dirty = true;
            }
            4 => {
                let pickers = ["fzf", "skim", "builtin"];
                let pidx = Select::with_theme(&theme)
                    .with_prompt("Picker")
                    .items(pickers)
                    .default(0)
                    .interact()
                    .map_err(wiz_err)?;
                config.ui.picker = match pidx {
                    1 => Picker::Skim,
                    2 => Picker::Builtin,
                    _ => Picker::Fzf,
                };
                dirty = true;
            }
            5 => {
                config.ui.stream = Confirm::with_theme(&theme)
                    .with_prompt("Stream replies?")
                    .default(config.ui.stream)
                    .interact()
                    .map_err(wiz_err)?;
                dirty = true;
            }
            6 => {
                config.ui.show_prompt_segment = Confirm::with_theme(&theme)
                    .with_prompt("Show prompt segment?")
                    .default(config.ui.show_prompt_segment)
                    .interact()
                    .map_err(wiz_err)?;
                dirty = true;
            }
            7 => {
                return Ok(if dirty { Some(config) } else { None });
            }
            _ => return Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(models: &[&str], default: &str) -> WizardDraft {
        WizardDraft {
            agent_name: "claude".to_string(),
            command: "claude-agent-acp".to_string(),
            models: models.iter().map(|s| s.to_string()).collect(),
            default_model: default.to_string(),
        }
    }

    #[test]
    fn parse_models_splits_on_space_and_comma() {
        assert_eq!(parse_models("a, b  c,,d"), vec!["a", "b", "c", "d"]);
        assert!(parse_models("   ").is_empty());
    }

    #[test]
    fn into_config_for_preset_draft_validates() {
        let c = draft(&["sonnet", "opus"], "sonnet").into_config();
        assert_eq!(c.default_agent, "claude");
        c.validate().expect("preset draft must validate");
    }

    #[test]
    fn into_config_for_custom_single_model_validates() {
        let mut d = draft(&["only"], "only");
        d.agent_name = "mine".to_string();
        d.command = "my-acp".to_string();
        let c = d.into_config();
        assert_eq!(c.default_agent, "mine");
        c.validate().expect("custom draft must validate");
    }

    #[test]
    fn into_agent_has_empty_passthrough() {
        let (name, agent) = draft(&["sonnet"], "sonnet").into_agent();
        assert_eq!(name, "claude");
        assert!(agent.extra.is_empty());
    }

    #[test]
    fn editor_style_mutation_preserves_passthrough_round_trip() {
        // Simulate what run_editor's actions do to an in-memory Config (add a
        // model, change a surfaced ui field) and confirm a write→load round
        // trip keeps the agent's opaque passthrough key (FR-008).
        let toml = r#"
default_agent = "codex"
[agents.codex]
command = "codex-acp"
models = ["gpt-5"]
default_model = "gpt-5"
api_key_env = "OPENAI_API_KEY"
"#;
        let mut config: Config = toml::from_str(toml).unwrap();
        config
            .agents
            .get_mut("codex")
            .unwrap()
            .models
            .push("gpt-5-thinking".to_string());
        config.ui.picker = Picker::Builtin;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        config.write(&path).expect("write");
        let reloaded = Config::load(&path).expect("reload");

        assert_eq!(
            reloaded.agents["codex"].passthrough().get("api_key_env"),
            Some(&"OPENAI_API_KEY".to_string())
        );
        assert!(
            reloaded.agents["codex"]
                .models
                .contains(&"gpt-5-thinking".to_string())
        );
        assert_eq!(reloaded.ui.picker, Picker::Builtin);
    }
}
