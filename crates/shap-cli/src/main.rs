//! `shap` — shell-native interface for ACP coding agents.
//!
//! Thin binary: parse CLI, init logging, dispatch to handlers, map results to
//! exit codes (0 success, 1 handled error, 2 usage — the latter via clap).

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
    match args.command {
        Command::Send { .. } => Err(unimplemented("send")),
        Command::Agent { .. } => Err(unimplemented("agent")),
        Command::Model { .. } => Err(unimplemented("model")),
        Command::Reasoning { .. } => Err(unimplemented("reasoning")),
        Command::New => Err(unimplemented("new")),
        Command::Status { .. } => Err(unimplemented("status")),
        Command::Commit { .. } => Err(unimplemented("commit")),
        Command::Run { .. } => Err(unimplemented("run")),
        Command::Read { .. } => Err(unimplemented("read")),
        Command::Doctor { .. } => Err(unimplemented("doctor")),
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
