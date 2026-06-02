//! Binary-side orchestration: load context and wire core handlers to the real
//! ACP client + renderer. Kept thin — all product logic lives in `shap-core`.

use std::io::{IsTerminal, Read};
use std::path::PathBuf;

use shap_agent::AcpClient;
use shap_agent::registry::Registry;
use shap_core::config::Config;
use shap_core::paths::{EnvVars, Paths};
use shap_core::state::ActiveState;
use shap_core::{Error, commands, doctor, output_capture, picker};
use shap_shell::render::Renderer;

/// Loaded runtime context shared by handlers.
pub struct Context {
    pub env: EnvVars,
    pub paths: Paths,
    pub config: Config,
    pub state: ActiveState,
    pub cwd: PathBuf,
}

impl Context {
    /// Resolve paths, load + validate config, load and reconcile state.
    pub fn load(
        config_override: Option<PathBuf>,
        cwd_override: Option<PathBuf>,
    ) -> Result<Self, Error> {
        let env = EnvVars::from_process();
        let paths = Paths::resolve(&env, config_override);
        let config = Config::load(paths.config())?;
        let mut state = ActiveState::load(&paths.state())?;
        if state.reconcile(&config) {
            let _ = state.save(&paths.state());
        }
        let cwd = cwd_override
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        Ok(Self {
            env,
            paths,
            config,
            state,
            cwd,
        })
    }

    fn sessions_dir(&self) -> PathBuf {
        self.config.sessions_dir(&self.paths, &self.env)
    }
}

/// `shap send` — resolve the agent, send the prompt, render the reply.
pub async fn send(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    prompt: &str,
) -> Result<(), Error> {
    let mut ctx = Context::load(config_override, cwd_override)?;
    let opts = Registry::new(&ctx.config).resolve(&ctx.state, ctx.cwd.clone())?;
    Registry::ensure_available(&opts)?;

    let sessions_dir = ctx.sessions_dir();
    let state_path = ctx.paths.state();
    let client = AcpClient::new();
    let mut renderer = Renderer::new(ctx.config.ui.stream);

    let result = {
        let mut on_chunk = |s: &str| renderer.chunk(s);
        commands::send(
            &opts,
            &ctx.config.files,
            &sessions_dir,
            &mut ctx.state,
            prompt,
            &client,
            &mut on_chunk,
        )
        .await
    };

    let outcome = result?;
    renderer.finish(&outcome.response);
    let _ = ctx.state.save(&state_path);
    Ok(())
}

/// `shap agent` — select the active agent (resets the model if it becomes
/// invalid for the new agent).
pub fn set_agent(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    name: Option<String>,
    force_picker: bool,
) -> Result<(), Error> {
    let mut ctx = Context::load(config_override, cwd_override)?;
    let kind = picker::resolve_from_path(ctx.config.ui.picker);
    let chosen = commands::set_agent(&ctx.config, &mut ctx.state, name, force_picker, kind)?;
    ctx.state.save(&ctx.paths.state())?;
    println!("agent: {chosen}");
    Ok(())
}

/// `shap model` — select the active agent's model.
pub fn set_model(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    name: Option<String>,
    force_picker: bool,
) -> Result<(), Error> {
    let mut ctx = Context::load(config_override, cwd_override)?;
    let kind = picker::resolve_from_path(ctx.config.ui.picker);
    let chosen = commands::set_model(&ctx.config, &mut ctx.state, name, force_picker, kind)?;
    ctx.state.save(&ctx.paths.state())?;
    println!("model: {chosen}");
    Ok(())
}

/// `shap reasoning` / `:effort` — select the reasoning effort.
pub fn set_reasoning(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    level: Option<String>,
    force_picker: bool,
) -> Result<(), Error> {
    let mut ctx = Context::load(config_override, cwd_override)?;
    let kind = picker::resolve_from_path(ctx.config.ui.picker);
    let chosen = commands::set_reasoning(&mut ctx.state, level, force_picker, kind)?;
    ctx.state.save(&ctx.paths.state())?;
    println!("reasoning: {chosen}");
    Ok(())
}

/// `shap new` — start a fresh session, preserving agent/model/reasoning.
pub fn new_session(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
) -> Result<(), Error> {
    let mut ctx = Context::load(config_override, cwd_override)?;
    let sessions_dir = ctx.sessions_dir();
    let id = commands::new_session(&ctx.config, &mut ctx.state, &sessions_dir)?;
    ctx.state.save(&ctx.paths.state())?;
    println!("new session: {id}");
    Ok(())
}

/// `shap status` — show the active agent/model/reasoning/session.
pub fn status(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    json: bool,
) -> Result<(), Error> {
    let ctx = Context::load(config_override, cwd_override)?;
    let status = commands::status(&ctx.state);
    if json {
        println!("{}", commands::status_json(&status)?);
    } else {
        let dash = |o: &Option<String>| o.clone().unwrap_or_else(|| "-".to_string());
        println!("agent:     {}", dash(&status.agent));
        println!("model:     {}", dash(&status.model));
        println!("reasoning: {}", dash(&status.reasoning));
        println!("session:   {}", dash(&status.session_id));
    }
    Ok(())
}

/// `shap run -- <cmd...>` — run a command, stream + capture its output for
/// `:read`, and exit with the child's code.
pub async fn run(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    argv: &[String],
) -> Result<i32, Error> {
    let ctx = Context::load(config_override, cwd_override)?;
    let result = commands::run(&ctx.cwd, argv).await?;
    let _ = output_capture::save(
        &ctx.paths.capture_output(),
        &ctx.paths.capture_meta(),
        &argv.join(" "),
        Some(result.exit_code),
        &result.output,
        ctx.config.history.max_output_bytes,
    );
    Ok(result.exit_code)
}

/// `shap read <prompt>` — send the prompt plus the last captured output (or
/// piped stdin) to the agent.
pub async fn read(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
    prompt: Option<String>,
) -> Result<(), Error> {
    let mut ctx = Context::load(config_override, cwd_override)?;
    let opts = Registry::new(&ctx.config).resolve(&ctx.state, ctx.cwd.clone())?;
    Registry::ensure_available(&opts)?;
    let sessions_dir = ctx.sessions_dir();
    let state_path = ctx.paths.state();

    // Pipe mode (stdin not a terminal) takes precedence over the stored capture.
    let (command, exit_code, output, truncated) = if !std::io::stdin().is_terminal() {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| Error::io("reading stdin", e))?;
        ("<pipe>".to_string(), None, buf, false)
    } else {
        let cap = output_capture::load(&ctx.paths.capture_output(), &ctx.paths.capture_meta())?;
        (cap.command, cap.exit_code, cap.output, cap.truncated)
    };

    let prompt_text = prompt.unwrap_or_default();
    let client = AcpClient::new();
    let mut renderer = Renderer::new(ctx.config.ui.stream);

    let result = {
        let mut on_chunk = |s: &str| renderer.chunk(s);
        commands::read(
            &opts,
            &sessions_dir,
            &mut ctx.state,
            &command,
            exit_code,
            &output,
            truncated,
            &prompt_text,
            &client,
            &mut on_chunk,
        )
        .await
    };

    let outcome = result?;
    renderer.finish(&outcome.response);
    let _ = ctx.state.save(&state_path);
    Ok(())
}

/// `shap commit --prefill-shell-buffer` — print a `git commit -am "…"` line for
/// the shell to insert. Never runs `git commit`. Prints nothing on stdout when
/// there is nothing to commit (a note goes to stderr); exits 0.
pub async fn commit(
    config_override: Option<PathBuf>,
    cwd_override: Option<PathBuf>,
) -> Result<(), Error> {
    let ctx = Context::load(config_override, cwd_override)?;
    let opts = Registry::new(&ctx.config).resolve(&ctx.state, ctx.cwd.clone())?;
    let client = AcpClient::new();
    // The generated message must not leak onto stdout (the shell captures only
    // the final `git commit` line), so chunks are dropped.
    let mut sink = |_: &str| {};
    match commands::commit(&opts, &client, &mut sink).await? {
        Some(line) => println!("{line}"),
        None => eprintln!("nothing to commit"),
    }
    Ok(())
}

/// `shap prompt-segment` — print the cached prompt segment. Reads only
/// state.json (no config, no agent) so the shell hook stays cheap.
pub fn prompt_segment(config_override: Option<PathBuf>) -> Result<(), Error> {
    let env = EnvVars::from_process();
    let paths = Paths::resolve(&env, config_override);
    let state = ActiveState::load(&paths.state())?;
    print!("{}", shap_shell::prompt::segment(&state));
    Ok(())
}

/// `shap config [--schema]` — print the config schema, or the resolved config
/// path.
pub fn config(config_override: Option<PathBuf>, schema: bool) -> Result<(), Error> {
    if schema {
        println!("{}", commands::config_schema()?);
    } else {
        let env = EnvVars::from_process();
        let paths = Paths::resolve(&env, config_override);
        println!("{}", paths.config().display());
    }
    Ok(())
}

/// `shap completions <shell>` — print a completion script for the shell.
pub fn completions(shell: clap_complete::Shell) {
    use clap::CommandFactory;
    let mut cmd = crate::cli::Cli::command();
    clap_complete::generate(shell, &mut cmd, "shap", &mut std::io::stdout());
}

/// `shap doctor` — validate the installation. Exits 0 if all critical checks
/// pass, else 1. A config error is reported as a failing check, not an abort.
pub fn doctor(
    config_override: Option<PathBuf>,
    _cwd_override: Option<PathBuf>,
    json: bool,
) -> Result<i32, Error> {
    let env = EnvVars::from_process();
    let paths = Paths::resolve(&env, config_override);
    let config = Config::load(paths.config());
    let state = ActiveState::load(&paths.state()).unwrap_or_default();
    let sessions_dir = config
        .as_ref()
        .map(|c| c.sessions_dir(&paths, &env))
        .unwrap_or_else(|_| paths.default_sessions_dir());

    let report = doctor::run(config.as_ref(), &state, &sessions_dir, &doctor::RealProbe);

    if json {
        println!("{}", commands::doctor_json(&report)?);
    } else {
        print!("{report}");
    }
    Ok(if report.ok() { 0 } else { 1 })
}
