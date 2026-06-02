//! `shap` — shell-native interface for ACP coding agents.
//!
//! Thin binary: parse CLI, init logging, dispatch to handlers, map results to
//! exit codes (0 success, 1 handled error, 2 usage — the latter via clap).

mod app;
mod cli;

use clap::Parser;
use cli::{Cli, Command};
use shap_core::Error;

#[tokio::main]
async fn main() {
    init_tracing();
    // clap exits with code 2 on usage errors before we get here.
    let args = Cli::parse();
    let code = match dispatch(args).await {
        Ok(()) => 0,
        Err(err) => {
            let report = miette::Report::new(err);
            eprintln!("{report:?}");
            1
        }
    };
    std::process::exit(code);
}

/// Route a parsed command to its handler. Handlers land per user story; until
/// then they report a clear "not implemented yet" diagnostic.
async fn dispatch(args: Cli) -> Result<(), Error> {
    let Cli {
        cwd,
        config,
        command,
    } = args;
    match command {
        Command::Send { prompt } => app::send(config, cwd, &prompt).await,
        Command::Agent { name, picker } => app::set_agent(config, cwd, name, picker),
        Command::Model { name, picker } => app::set_model(config, cwd, name, picker),
        Command::Reasoning { level, picker } => app::set_reasoning(config, cwd, level, picker),
        Command::New => Err(unimplemented("new")),
        Command::Status { .. } => Err(unimplemented("status")),
        Command::Commit { .. } => Err(unimplemented("commit")),
        Command::Run { .. } => Err(unimplemented("run")),
        Command::Read { .. } => Err(unimplemented("read")),
        Command::Doctor { .. } => Err(unimplemented("doctor")),
        Command::PromptSegment => app::prompt_segment(config),
    }
}

fn unimplemented(command: &'static str) -> Error {
    Error::Unimplemented { command }
}

fn init_tracing() {
    use tracing_subscriber::{EnvFilter, fmt};
    let filter = EnvFilter::try_from_env("SHAP_LOG")
        .or_else(|_| EnvFilter::try_from_default_env())
        .unwrap_or_else(|_| EnvFilter::new("warn"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .without_time()
        .init();
}
