# Configuration

`shap` reads a single user-editable TOML file. The tool writes it only when you ask it to — via the
first-run setup wizard or `shap config edit` (see [Setup wizard & editing](#setup-wizard--editing)).
It never rewrites the file silently or on the hot path.

## Location

| Purpose | Default | Override |
|---------|---------|----------|
| Config  | `${XDG_CONFIG_HOME:-~/.config}/shap/config.toml` | `--config <path>` / `SHAP_CONFIG` |
| State   | `${XDG_DATA_HOME:-~/.local/share}/shap/state.json` | `SHAP_DATA_DIR` |
| Sessions | `<data>/shap/sessions/` | `[history].dir` |
| Capture | `<data>/shap/last-command-output.txt` | `SHAP_DATA_DIR` |

On macOS the XDG layout is used (not the Apple container paths) for parity with Linux.

## Example

```toml
default_agent = "codex"

[agents.codex]
command = "codex-acp"
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"

[agents.claude]
command = "claude-agent-acp"
models = ["sonnet", "opus"]
default_model = "sonnet"

[ui]
stream = true              # stream replies vs. spinner-then-final
picker = "fzf"             # fzf | skim | builtin (falls back at runtime)
show_prompt_segment = true
# prompt_icon = ""        # Nerd Font glyph; unset → "shap" prefix

[history]
dir = "~/.local/share/shap/sessions"
max_output_bytes = 200000  # cap on captured :run output sent to the agent

[files]
max_file_bytes = 200000    # cap on a single @file inclusion
respect_gitignore = true
```

## Fields

### Top level
- `default_agent` (string, required) — must be a key under `[agents]`. Used when no agent is selected.
- `[agents.<name>]` (table, at least one required) — see below.

### `[agents.<name>]`
- `command` (string, required) — launch command for the external ACP process. Validated by `:doctor`.
- `models` (array, non-empty) — the only valid models for this agent.
- `default_model` (string) — must be a member of `models`.
- Any other keys are preserved as opaque agent-specific passthrough (see [agents.md](./agents.md)).

### `[ui]`
- `stream` (bool, default `true`)
- `picker` (`fzf` | `skim` | `builtin`, default `fzf`)
- `show_prompt_segment` (bool, default `true`)
- `prompt_icon` (string, optional) — Nerd Font glyph shown instead of `shap` in the prompt segment. Unset renders the plain `shap` prefix.

### `[history]`
- `dir` (path, default under the data dir) — `~` and `$XDG_*` are expanded.
- `max_output_bytes` (int > 0, default `200000`)

### `[files]`
- `max_file_bytes` (int > 0, default `200000`)
- `respect_gitignore` (bool, default `true`)

## Validation

Validation runs on load and fails with an actionable diagnostic (never a panic):
`default_agent` must be configured; each agent's `models` must be non-empty and contain its
`default_model`; byte limits must be `> 0`. The same validation runs before any wizard/editor write,
so `shap` never writes an invalid config.

## Setup wizard & editing

The config can be created and changed interactively — no hand-editing TOML required.

- **First run**: when a command needs config and none exists, `shap` offers a short setup wizard
  (pick an agent preset or a custom command, choose models and a default, accept UI defaults). On
  finish it writes a validated `config.toml` and continues. This only happens on an interactive
  terminal; in scripts, pipes, or the shell prompt hook `shap` prints setup instructions and exits
  non-zero instead of prompting.
- **Interactive editor**: `shap config edit` (or bare `shap config` on a terminal) walks you through
  changing the default agent, an agent's models/default model, the picker, streaming, and the prompt
  segment. Changes are re-validated and written atomically; cancelling or making no change leaves the
  file untouched.
- **Raw file editor**: `shap config open` opens the config TOML directly in your `$VISUAL` or
  `$EDITOR` (falling back to `vim`, `vi`, then `nano`). After the editor exits `shap` validates the
  file and reports any errors. If no config exists yet, the setup wizard runs first to create one.

Both write paths preserve agent-specific passthrough keys (e.g. `api_key_env`). The interactive
editor does **not** preserve comments or your original key ordering — the file is re-emitted as
canonical TOML. Use `shap config open` if you want to keep hand-maintained comments.

Non-interactive lookups stay script-friendly:

```sh
shap config path     # print the resolved config path
shap config          # same as `config path` when stdin is not a terminal
```

## JSON schema

A machine-readable schema is generated from the Rust types:

```sh
shap config --schema
```

## State (`state.json`)

Machine-written, updated atomically. Holds the active agent/model/reasoning/session and last cwd.
Selections that no longer exist in the config are dropped on the next read (and repaired on the next
selection). It is safe to delete — `shap` treats a missing file as a fresh install.

## See also

- [Getting started](./getting-started.md) · [Agents](./agents.md) ·
  [Documentation index](./index.md)
