//! Interactive selection via an external picker (fzf/skim) or a built-in
//! fallback (dialoguer).
//!
//! The configured [`Picker`] is a preference; resolution falls back through
//! `fzf → skim → builtin` based on what is actually on PATH (research D6).
//! When no terminal is available, selection returns an actionable error telling
//! the user to pass the value explicitly.

use std::io::{IsTerminal, Write};
use std::process::{Command, Stdio};

use crate::config::Picker;
use crate::error::{Error, Result};

/// The picker actually used at runtime, after fallback resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Fzf,
    Skim,
    Builtin,
}

/// Resolve the picker preference against availability. Pure (testable): the
/// caller supplies whether `fzf`/`skim` are present.
pub fn resolve(preference: Picker, has_fzf: bool, has_skim: bool) -> PickerKind {
    match preference {
        Picker::Builtin => PickerKind::Builtin,
        Picker::Fzf => first_available(&[(PickerKind::Fzf, has_fzf), (PickerKind::Skim, has_skim)]),
        Picker::Skim => {
            first_available(&[(PickerKind::Skim, has_skim), (PickerKind::Fzf, has_fzf)])
        }
    }
}

fn first_available(prefs: &[(PickerKind, bool)]) -> PickerKind {
    prefs
        .iter()
        .find(|(_, available)| *available)
        .map(|(kind, _)| *kind)
        .unwrap_or(PickerKind::Builtin)
}

/// Resolve using the live PATH.
pub fn resolve_from_path(preference: Picker) -> PickerKind {
    resolve(
        preference,
        which::which("fzf").is_ok(),
        which::which("sk").is_ok(),
    )
}

/// Present `items` and return the chosen one. `command` names the originating
/// subcommand, used in the non-interactive error's remediation.
pub fn select(kind: PickerKind, command: &str, items: &[String]) -> Result<String> {
    if items.is_empty() {
        return Err(Error::PickerEmpty);
    }
    match kind {
        PickerKind::Fzf => run_filter("fzf", command, items),
        PickerKind::Skim => run_filter("sk", command, items),
        PickerKind::Builtin => builtin_select(command, items),
    }
}

fn run_filter(bin: &str, command: &str, items: &[String]) -> Result<String> {
    let mut child = Command::new(bin)
        .arg("--prompt")
        .arg(format!("{command}> "))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| Error::PickerFailed {
            detail: format!("could not launch {bin}: {e}"),
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(items.join("\n").as_bytes());
        // stdin dropped here, closing the pipe so the picker can proceed.
    }

    let output = child.wait_with_output().map_err(|e| Error::PickerFailed {
        detail: format!("{bin} failed: {e}"),
    })?;

    if !output.status.success() {
        return Err(Error::PickerFailed {
            detail: "no selection made".to_string(),
        });
    }
    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        Err(Error::PickerFailed {
            detail: "no selection made".to_string(),
        })
    } else {
        Ok(selected)
    }
}

fn builtin_select(command: &str, items: &[String]) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        return Err(Error::NonInteractivePicker {
            command: command.to_string(),
        });
    }
    let index = dialoguer::Select::new()
        .with_prompt(command)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| Error::PickerFailed {
            detail: e.to_string(),
        })?;
    Ok(items[index].clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_prefers_configured_then_falls_back() {
        // fzf preferred, both present → fzf
        assert_eq!(resolve(Picker::Fzf, true, true), PickerKind::Fzf);
        // fzf preferred, only skim → skim
        assert_eq!(resolve(Picker::Fzf, false, true), PickerKind::Skim);
        // fzf preferred, neither → builtin
        assert_eq!(resolve(Picker::Fzf, false, false), PickerKind::Builtin);
        // skim preferred, only fzf → fzf
        assert_eq!(resolve(Picker::Skim, true, false), PickerKind::Fzf);
        // skim preferred, skim present → skim
        assert_eq!(resolve(Picker::Skim, true, true), PickerKind::Skim);
        // builtin always builtin
        assert_eq!(resolve(Picker::Builtin, true, true), PickerKind::Builtin);
    }

    #[test]
    fn empty_items_error() {
        let err = select(PickerKind::Builtin, "agent", &[]).unwrap_err();
        assert!(matches!(err, Error::PickerEmpty), "{err:?}");
    }
}
