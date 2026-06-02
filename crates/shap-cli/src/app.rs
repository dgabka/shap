//! Binary-side orchestration: load context and wire core handlers to the real
//! ACP client + renderer. Kept thin — all product logic lives in `shap-core`.

use std::path::PathBuf;

use shap_agent::AcpClient;
use shap_agent::registry::Registry;
use shap_core::config::Config;
use shap_core::paths::{EnvVars, Paths};
use shap_core::state::ActiveState;
use shap_core::{Error, commands, picker};
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

/// `shap prompt-segment` — print the cached prompt segment. Reads only
/// state.json (no config, no agent) so the shell hook stays cheap.
pub fn prompt_segment(config_override: Option<PathBuf>) -> Result<(), Error> {
    let env = EnvVars::from_process();
    let paths = Paths::resolve(&env, config_override);
    let state = ActiveState::load(&paths.state())?;
    print!("{}", shap_shell::prompt::segment(&state));
    Ok(())
}
