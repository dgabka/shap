# Contract: `config` Command Surface & Wizard Behavior

This contract defines the user-facing CLI behavior added/changed by this feature. The zsh layer adds
no semantics on top (consistent with `crates/shap-cli` being the contract surface).

## 1. First-run wizard (implicit, on any config-requiring command)

**Trigger**: a command that calls `Context::load` runs, the resolved `config.toml` does not exist,
and `stdin` is a TTY.

**Commands in scope** (all funnel through `Context::load`): `send`, `agent`, `model`, `reasoning`,
`new`, `status`, `commit`, `run`, `read`.

**Out of scope**: `doctor` (reports missing config as a failing check, no wizard), `completions`,
`prompt-segment` (never loads config), and any invocation where `stdin` is not a TTY.

| Precondition | Behavior | Exit |
|--------------|----------|------|
| No config, stdin is a TTY | Offer wizard (`Confirm`). On accept → run prompts → validate → atomic write → re-load → continue the original command. | original command's code |
| No config, stdin is a TTY, user declines or cancels mid-wizard | No file written. Print setup guidance (same content as the non-interactive `ConfigNotFound` help). | non-zero |
| No config, stdin **not** a TTY | No prompt. Print the existing `ConfigNotFound` diagnostic. | non-zero (unchanged) |
| Config exists | Wizard never triggers; behavior unchanged. | per command |

**Wizard prompt sequence** (target: ≤ ~5 prompts for the simple case, SC-002):

1. `Confirm`: "No config found. Set one up now?" (decline ⇒ guidance + non-zero exit).
2. `Select`: choose an agent preset (e.g. `codex`, `claude`) or `custom`.
   - preset ⇒ pre-fills agent name + command + starter models.
   - `custom` ⇒ `Input` agent name, `Input` launch command.
3. `Input`: models (space/comma-separated); must be non-empty (re-prompt if empty).
4. `Select`: default model from the entered models.
5. (single agent ⇒ it is the default agent automatically) `Confirm`: accept UI defaults
   (stream on, picker fzf, prompt segment on) or step through them.
6. Show the resulting config summary, `Confirm` write.

**Postconditions on success**: a `config.toml` exists at the resolved path that passes
`Config::validate()`; the originally requested command proceeds.

## 2. `shap config` subcommand group

Backward compatibility is mandatory (FR-012): existing scripted invocations must keep working.

| Invocation | Behavior | Notes |
|------------|----------|-------|
| `shap config` (stdin **not** TTY) | Print the resolved config path (today's no-flag behavior). | unchanged for scripts |
| `shap config` (stdin TTY) | Open the interactive editor (§3). | new default for humans |
| `shap config path` | Print the resolved config path. | explicit, always non-interactive |
| `shap config --schema` / `shap config schema` | Print the generated JSON schema. | unchanged output |
| `shap config edit` | Open the interactive editor (§3); requires a TTY. | |
| `shap config edit` (stdin **not** TTY) | Error: editing requires an interactive terminal. | non-zero, no write |

> Implementation note: whether `schema`/`path` are subcommands or flags is an implementation choice;
> the **contract** is that `shap config` (non-TTY) prints the path and `shap config --schema` prints
> the schema, both exactly as today.

## 3. Interactive editor (`shap config edit`, or bare `shap config` on a TTY)

**Precondition**: a config file exists and loads/validates. If it is missing, fall through to the
first-run wizard path (create). If it exists but fails to parse/validate, surface the existing
parse/validation diagnostic (repair-by-editor of a broken file is best-effort, not guaranteed).

**Behavior**:

- Load the current `Config`, present a menu of edit actions (see data-model `EditAction`).
- Apply changes to an in-memory clone.
- On **Save**: run `Config::validate()`; if it passes, atomic-write; if it fails, show the diagnostic
  and keep the existing file (FR-007).
- On **Cancel / no changes**: leave the existing file byte-for-byte unchanged (FR-009).
- Per-agent passthrough keys (`extra`) are preserved across the edit (FR-008).

| Scenario | Result | Exit |
|----------|--------|------|
| Change a setting, save, valid | File updated atomically; new value effective next command. | 0 |
| Change makes config invalid, save | Rejected with diagnostic; existing file preserved. | non-zero |
| No change / cancel | File unchanged. | 0 |
| Save fails (permission/disk) | `ConfigWriteFailed { path, .. }` diagnostic naming the path; no partial file. | non-zero |

## 4. Error / diagnostic contract

- Reuse `Error::ConfigNotFound` help text for the non-interactive and declined-wizard guidance.
- New `Error::ConfigWriteFailed { path, source }` — miette diagnostic with `code(shap::config::write)`
  and `help` naming the attempted path and a next step. Never a panic (FR-014).
- Non-interactive editor invocation → an actionable "requires a terminal" error (mirrors
  `Error::NonInteractivePicker` style).

## 5. Invariants (must hold for every path)

- **INV-1**: The tool never writes config without an explicit user action (accepting the wizard or
  confirming a save). No silent writes. (Constitution VII)
- **INV-2**: A written config always passes `Config::validate()`. (FR-004/FR-007, SC-003)
- **INV-3**: Cancellation/decline/EOF never leaves a partial file and never corrupts an existing one.
  (FR-005/FR-009, SC-004)
- **INV-4**: No prompting when `stdin` is not a TTY; the `prompt-segment` path never triggers a
  wizard or write. (FR-010/FR-011, SC-005)
- **INV-5**: `shap config` (non-TTY) and `shap config --schema` produce their current outputs.
  (FR-012)
