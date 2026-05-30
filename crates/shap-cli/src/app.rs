//! Binary-side orchestration: load context and wire core handlers to the real
//! ACP client + renderer. Kept thin — all product logic lives in `shap-core`.

use std::path::PathBuf;

use shap_agent::AcpClient;
use shap_agent::registry::Registry;
use shap_core::config::Config;
use shap_core::paths::{EnvVars, Paths};
use shap_core::state::ActiveState;
use shap_core::{Error, commands};
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
