//! Domain error type for shap.
//!
//! Every variant carries an actionable `miette` diagnostic: the help text
//! names a concrete next step (often "run `shap doctor`"). Errors are never
//! panics — config and agent problems surface as diagnostics (FR-028/029/030).

use std::path::PathBuf;

use miette::Diagnostic;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("no config file at {path}")]
    #[diagnostic(
        code(shap::config::missing),
        help(
            "create it with at least one agent, e.g.:\n\n  default_agent = \"codex\"\n\n  [agents.codex]\n  command = \"codex-acp\"\n  models = [\"gpt-5\"]\n  default_model = \"gpt-5\"\n\nthen run `shap doctor`."
        )
    )]
    ConfigNotFound { path: PathBuf },

    #[error("could not parse config at {path}")]
    #[diagnostic(
        code(shap::config::parse),
        help("fix the TOML syntax, then run `shap doctor`.")
    )]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("failed to write config at {path}")]
    #[diagnostic(
        code(shap::config::write),
        help("check the directory exists and is writable, then retry.")
    )]
    ConfigWriteFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("no agents configured")]
    #[diagnostic(
        code(shap::config::no_agents),
        help("add at least one `[agents.<name>]` table to your config, then run `shap doctor`.")
    )]
    NoAgentConfigured,

    #[error("default_agent \"{agent}\" is not configured")]
    #[diagnostic(
        code(shap::config::unknown_default_agent),
        help("set default_agent to one of: {configured}")
    )]
    UnknownDefaultAgent { agent: String, configured: String },

    #[error("agent \"{agent}\" has an empty `models` list")]
    #[diagnostic(
        code(shap::config::empty_models),
        help("list at least one model under [agents.{agent}].")
    )]
    AgentEmptyModels { agent: String },

    #[error("agent \"{agent}\" default_model \"{default_model}\" is not in its `models`")]
    #[diagnostic(
        code(shap::config::bad_default_model),
        help("set default_model to one of: {models}")
    )]
    DefaultModelNotInModels {
        agent: String,
        default_model: String,
        models: String,
    },

    #[error("{field} must be greater than 0")]
    #[diagnostic(code(shap::config::non_positive_limit))]
    NonPositiveByteLimit { field: &'static str },

    #[error("unknown agent \"{name}\"")]
    #[diagnostic(code(shap::agent::unknown), help("configured agents: {configured}"))]
    UnknownAgent { name: String, configured: String },

    #[error("no active agent selected")]
    #[diagnostic(
        code(shap::agent::none_active),
        help("select one with `:agent` (or `shap agent <name>`).")
    )]
    NoActiveAgent,

    #[error("model \"{model}\" is not valid for agent \"{agent}\"")]
    #[diagnostic(code(shap::model::invalid), help("valid models for {agent}: {models}"))]
    ModelNotForAgent {
        model: String,
        agent: String,
        models: String,
    },

    #[error("agent command \"{command}\" was not found on PATH")]
    #[diagnostic(
        code(shap::agent::missing_command),
        help("install it or fix the `command` in your config, then run `shap doctor`.")
    )]
    AgentCommandMissing { command: String },

    #[error("failed to launch agent \"{agent}\" ({command})")]
    #[diagnostic(
        code(shap::agent::spawn),
        help("run `shap doctor` to check the agent command.")
    )]
    AgentSpawn {
        agent: String,
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("agent \"{agent}\" became unavailable: {reason}")]
    #[diagnostic(code(shap::agent::unavailable), help("run `shap doctor`, then retry."))]
    AgentUnavailable { agent: String, reason: String },

    #[error("agent protocol error: {0}")]
    #[diagnostic(code(shap::agent::protocol))]
    AgentProtocol(String),

    #[error("not inside a Git repository")]
    #[diagnostic(
        code(shap::git::not_a_repo),
        help("run this from within a Git working tree.")
    )]
    NotAGitRepo,

    #[error("nothing to commit")]
    #[diagnostic(
        code(shap::git::nothing_to_commit),
        help("stage or modify files first.")
    )]
    NothingToCommit,

    #[error("git is not available on PATH")]
    #[diagnostic(code(shap::git::missing), help("install git, then run `shap doctor`."))]
    GitUnavailable,

    #[error("nothing captured")]
    #[diagnostic(
        code(shap::capture::empty),
        help("run `:run <cmd>` first, or pipe output into `:read`.")
    )]
    NoCapturedOutput,

    #[error("file too large: {path} is {bytes} bytes (max {max})")]
    #[diagnostic(
        code(shap::files::too_large),
        help("raise [files].max_file_bytes or reference a smaller file.")
    )]
    FileTooLarge { path: PathBuf, bytes: u64, max: u64 },

    #[error("\"{level}\" is not a valid reasoning level")]
    #[diagnostic(code(shap::reasoning::invalid), help("choose one of: {valid}"))]
    InvalidReasoning { level: String, valid: String },

    #[error("cannot open an interactive picker (no terminal)")]
    #[diagnostic(
        code(shap::picker::non_interactive),
        help("pass a value explicitly, e.g. `shap {command} <value>`.")
    )]
    NonInteractivePicker { command: String },

    #[error("no suitable editor found")]
    #[diagnostic(
        code(shap::config::no_editor),
        help("set $EDITOR or $VISUAL, or install vim/vi/nano.")
    )]
    EditorNotFound,

    #[error("nothing to choose from")]
    #[diagnostic(code(shap::picker::empty))]
    PickerEmpty,

    #[error("selection cancelled")]
    #[diagnostic(code(shap::picker::cancelled), help("{detail}"))]
    PickerFailed { detail: String },

    #[error("{command} is not implemented yet")]
    #[diagnostic(code(shap::unimplemented))]
    Unimplemented { command: &'static str },

    #[error("{context}")]
    #[diagnostic(code(shap::io))]
    Io {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

impl Error {
    /// Wrap an I/O error with human context.
    pub fn io(context: impl Into<String>, source: std::io::Error) -> Self {
        Error::Io {
            context: context.into(),
            source,
        }
    }
}
