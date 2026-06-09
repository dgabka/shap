# Configuration

`shap` reads a single user-editable TOML file. The tool never rewrites it.

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

### `[history]`
- `dir` (path, default under the data dir) — `~` and `$XDG_*` are expanded.
- `max_output_bytes` (int > 0, default `200000`)

### `[files]`
- `max_file_bytes` (int > 0, default `200000`)
- `respect_gitignore` (bool, default `true`)

## Validation

Validation runs on load and fails with an actionable diagnostic (never a panic):
`default_agent` must be configured; each agent's `models` must be non-empty and contain its
`default_model`; byte limits must be `> 0`. A missing config prints setup instructions.

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
