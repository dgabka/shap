//! clap command-line definitions.
//!
//! This is the contract surface (see `contracts/cli-commands.md`): the Zsh
//! layer maps `:` commands to these subcommands and adds nothing semantically.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "shap",
    version,
    about = "Shell-native interface for ACP coding agents",
    propagate_version = true
)]
pub struct Cli {
    /// Working directory (the shell forwards the current dir).
    #[arg(long, global = true, value_name = "PATH")]
    pub cwd: Option<PathBuf>,

    /// Path to config.toml (overrides SHAP_CONFIG / XDG).
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Send a prompt to the active agent.
    Send {
        /// Prompt text (may contain `@file` references).
        prompt: String,
    },

    /// Select the active agent (no name → picker).
    Agent {
        name: Option<String>,
        /// Force the picker even when a name is given.
        #[arg(long)]
        picker: bool,
    },

    /// Select the active model (active agent's models only).
    Model {
        name: Option<String>,
        #[arg(long)]
        picker: bool,
    },

    /// Select reasoning effort (also reachable via `:effort`).
    Reasoning {
        level: Option<String>,
        #[arg(long)]
        picker: bool,
    },

    /// Start a new session, keeping agent/model/reasoning.
    New,

    /// Show the active agent/model/reasoning/session.
    Status {
        /// Emit machine-readable JSON (consumed by the prompt segment).
        #[arg(long)]
        json: bool,
    },

    /// Generate a commit message and print a `git commit` line.
    ///
    /// Never executes `git commit` — the shell inserts the line into the buffer.
    Commit {
        #[arg(long)]
        prefill_shell_buffer: bool,
    },

    /// Run a command and capture its combined output.
    Run {
        /// The command to run, after `--` (e.g. `shap run -- cargo test`).
        #[arg(trailing_var_arg = true, allow_hyphen_values = true, num_args = 1.., value_name = "COMMAND")]
        command: Vec<String>,
    },

    /// Send a prompt plus the last captured output (or piped stdin).
    Read { prompt: Option<String> },

    /// Validate the installation and configured agents.
    Doctor {
        #[arg(long)]
        json: bool,
    },

    /// Inspect or edit configuration.
    ///
    /// With no subcommand: opens the interactive editor on a terminal, else
    /// prints the resolved config path (back-compatible for scripts).
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,
        /// Print the configuration JSON schema (generated from the types).
        #[arg(long)]
        schema: bool,
    },

    /// Generate shell completions for the given shell.
    Completions { shell: clap_complete::Shell },

    /// Print the prompt segment from cached state (used by the shell hook).
    ///
    /// Cheap by design: reads only state.json, never the config or an agent.
    #[command(hide = true)]
    PromptSegment,
}

/// Subcommands of `shap config`.
#[derive(Debug, Subcommand)]
pub enum ConfigAction {
    /// Print the resolved config path.
    Path,
    /// Print the configuration JSON schema.
    Schema,
    /// Interactively edit the config (requires a terminal).
    Edit,
}
