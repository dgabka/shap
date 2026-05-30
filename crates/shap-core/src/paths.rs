//! Path resolution for config, state, sessions, and captured output.
//!
//! Resolution is a pure function of [`EnvVars`] so it can be tested without
//! mutating the process environment (which is `unsafe` under edition 2024 and
//! racy across parallel tests). On macOS we deliberately use the XDG layout
//! (not the Apple container paths) for parity with Linux.

use std::path::{Path, PathBuf};

/// Environment variables relevant to path resolution, captured up front.
#[derive(Debug, Clone, Default)]
pub struct EnvVars {
    pub home: Option<PathBuf>,
    pub xdg_config_home: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub shap_config: Option<PathBuf>,
    pub shap_data_dir: Option<PathBuf>,
}

impl EnvVars {
    /// Read the relevant variables from the live process environment. Empty
    /// values are treated as unset.
    pub fn from_process() -> Self {
        let var = |k: &str| {
            std::env::var_os(k)
                .filter(|v| !v.is_empty())
                .map(PathBuf::from)
        };
        Self {
            home: var("HOME"),
            xdg_config_home: var("XDG_CONFIG_HOME"),
            xdg_data_home: var("XDG_DATA_HOME"),
            shap_config: var("SHAP_CONFIG"),
            shap_data_dir: var("SHAP_DATA_DIR"),
        }
    }
}

/// Resolved base locations for shap's on-disk files.
#[derive(Debug, Clone)]
pub struct Paths {
    config: PathBuf,
    data_dir: PathBuf,
}

impl Paths {
    /// Resolve paths from the environment, honouring an explicit `--config`
    /// override (highest precedence) and the `SHAP_CONFIG`/`SHAP_DATA_DIR` and
    /// `XDG_*` variables.
    pub fn resolve(env: &EnvVars, config_override: Option<PathBuf>) -> Self {
        let data_dir = if let Some(d) = &env.shap_data_dir {
            d.clone()
        } else if let Some(x) = &env.xdg_data_home {
            x.join("shap")
        } else {
            home_join(env, &[".local", "share", "shap"])
        };

        let config = if let Some(c) = config_override {
            c
        } else if let Some(c) = &env.shap_config {
            c.clone()
        } else if let Some(x) = &env.xdg_config_home {
            x.join("shap").join("config.toml")
        } else {
            home_join(env, &[".config", "shap", "config.toml"])
        };

        Self { config, data_dir }
    }

    /// Convenience constructor reading the live process environment.
    pub fn from_process(config_override: Option<PathBuf>) -> Self {
        Self::resolve(&EnvVars::from_process(), config_override)
    }

    pub fn config(&self) -> &Path {
        &self.config
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn state(&self) -> PathBuf {
        self.data_dir.join("state.json")
    }

    pub fn capture_output(&self) -> PathBuf {
        self.data_dir.join("last-command-output.txt")
    }

    pub fn capture_meta(&self) -> PathBuf {
        self.data_dir.join("last-command-output.json")
    }

    pub fn default_sessions_dir(&self) -> PathBuf {
        self.data_dir.join("sessions")
    }
}

fn home_join(env: &EnvVars, parts: &[&str]) -> PathBuf {
    let mut p = env.home.clone().unwrap_or_else(|| PathBuf::from("."));
    for part in parts {
        p.push(part);
    }
    p
}

/// Expand a user-supplied path: a leading `~`/`~/`, and `$VAR` / `${VAR}`
/// references. Only `HOME` and `XDG_*` are recognised; unknown variables are
/// left literal so a typo is visible rather than silently emptied.
pub fn expand_path(raw: &str, env: &EnvVars) -> PathBuf {
    if raw == "~" {
        if let Some(h) = &env.home {
            return h.clone();
        }
    } else if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(h) = &env.home {
            return h.join(rest);
        }
    }
    PathBuf::from(substitute_vars(raw, env))
}

fn substitute_vars(input: &str, env: &EnvVars) -> String {
    let lookup = |name: &str| -> Option<String> {
        let value = match name {
            "HOME" => env.home.as_ref(),
            "XDG_CONFIG_HOME" => env.xdg_config_home.as_ref(),
            "XDG_DATA_HOME" => env.xdg_data_home.as_ref(),
            _ => None,
        };
        value.map(|p| p.to_string_lossy().into_owned())
    };

    let mut out = String::with_capacity(input.len());
    let mut chars = input.char_indices().peekable();
    while let Some((_, c)) = chars.next() {
        if c != '$' {
            out.push(c);
            continue;
        }
        // `${NAME}`
        if matches!(chars.peek(), Some((_, '{'))) {
            chars.next();
            let mut name = String::new();
            let mut closed = false;
            for (_, nc) in chars.by_ref() {
                if nc == '}' {
                    closed = true;
                    break;
                }
                name.push(nc);
            }
            match (closed, lookup(&name)) {
                (true, Some(v)) => out.push_str(&v),
                (true, None) => {
                    out.push_str("${");
                    out.push_str(&name);
                    out.push('}');
                }
                (false, _) => {
                    out.push_str("${");
                    out.push_str(&name);
                }
            }
        } else {
            // `$NAME` (alnum + underscore)
            let mut name = String::new();
            while let Some((_, nc)) = chars.peek() {
                if nc.is_ascii_alphanumeric() || *nc == '_' {
                    name.push(*nc);
                    chars.next();
                } else {
                    break;
                }
            }
            match lookup(&name) {
                Some(v) if !name.is_empty() => out.push_str(&v),
                _ => {
                    out.push('$');
                    out.push_str(&name);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> EnvVars {
        EnvVars {
            home: Some(PathBuf::from("/home/u")),
            xdg_config_home: None,
            xdg_data_home: None,
            shap_config: None,
            shap_data_dir: None,
        }
    }

    #[test]
    fn default_layout_uses_home() {
        let p = Paths::resolve(&env(), None);
        assert_eq!(p.config(), Path::new("/home/u/.config/shap/config.toml"));
        assert_eq!(p.data_dir(), Path::new("/home/u/.local/share/shap"));
        assert_eq!(
            p.state(),
            PathBuf::from("/home/u/.local/share/shap/state.json")
        );
        assert_eq!(
            p.default_sessions_dir(),
            PathBuf::from("/home/u/.local/share/shap/sessions")
        );
    }

    #[test]
    fn xdg_overrides_home() {
        let mut e = env();
        e.xdg_config_home = Some(PathBuf::from("/cfg"));
        e.xdg_data_home = Some(PathBuf::from("/data"));
        let p = Paths::resolve(&e, None);
        assert_eq!(p.config(), Path::new("/cfg/shap/config.toml"));
        assert_eq!(p.data_dir(), Path::new("/data/shap"));
    }

    #[test]
    fn shap_env_overrides_xdg() {
        let mut e = env();
        e.xdg_config_home = Some(PathBuf::from("/cfg"));
        e.xdg_data_home = Some(PathBuf::from("/data"));
        e.shap_config = Some(PathBuf::from("/explicit/c.toml"));
        e.shap_data_dir = Some(PathBuf::from("/explicit/data"));
        let p = Paths::resolve(&e, None);
        assert_eq!(p.config(), Path::new("/explicit/c.toml"));
        assert_eq!(p.data_dir(), Path::new("/explicit/data"));
    }

    #[test]
    fn cli_override_beats_everything() {
        let mut e = env();
        e.shap_config = Some(PathBuf::from("/env/c.toml"));
        let p = Paths::resolve(&e, Some(PathBuf::from("/flag/c.toml")));
        assert_eq!(p.config(), Path::new("/flag/c.toml"));
    }

    #[test]
    fn expand_tilde() {
        assert_eq!(expand_path("~", &env()), PathBuf::from("/home/u"));
        assert_eq!(expand_path("~/x/y", &env()), PathBuf::from("/home/u/x/y"));
        // `~user` is not expanded (only `~` / `~/`).
        assert_eq!(expand_path("~bob/x", &env()), PathBuf::from("~bob/x"));
    }

    #[test]
    fn expand_env_vars() {
        let mut e = env();
        e.xdg_data_home = Some(PathBuf::from("/data"));
        assert_eq!(
            expand_path("$XDG_DATA_HOME/shap", &e),
            PathBuf::from("/data/shap")
        );
        assert_eq!(
            expand_path("${XDG_DATA_HOME}/s", &e),
            PathBuf::from("/data/s")
        );
        assert_eq!(expand_path("$HOME/p", &e), PathBuf::from("/home/u/p"));
        // Unknown variable is left literal.
        assert_eq!(expand_path("$NOPE/x", &e), PathBuf::from("$NOPE/x"));
    }
}
