# Contract: Configuration & State Files

**Feature**: `001-shell-agent-acp` | **Date**: 2026-05-30

Defines the on-disk contract for user config and machine state. Field semantics and validation live in
[data-model.md](../data-model.md); this file pins the concrete file format and defaults. A JSON Schema
for the config is generated from the Rust types via `schemars` and published in `docs/config.md`.

## config.toml — `~/.config/shap/config.toml`

User-editable. Read-only from the tool's perspective (the tool never rewrites it).

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
stream = true            # default true
picker = "fzf"           # fzf | skim | builtin ; default fzf, falls back at runtime
show_prompt_segment = true  # default true

[history]
dir = "~/.local/share/shap/sessions"  # ~ expanded
capture_last_output = false           # reserved; MVP capture is explicit
max_output_bytes = 200000             # > 0

[files]
max_file_bytes = 200000   # > 0
respect_gitignore = true
```

**Validation (on load)** — failures produce an actionable `miette` diagnostic, never a panic:
- `default_agent` MUST be a key under `[agents]`.
- Each agent: non-empty `models`; `default_model` ∈ `models`.
- `[ui].picker` ∈ {`fzf`,`skim`,`builtin`}.
- `max_output_bytes`, `max_file_bytes` > 0.
- Unknown agent-specific keys under `[agents.<name>]` are preserved as opaque passthrough (FR-022),
  not rejected.
- Missing file → FR-029 setup instructions (not an error dump).

## state.json — `~/.local/share/shap/state.json`

Machine-written. Created on first selection; updated atomically (write temp + rename).

```json
{
  "active_agent": "codex",
  "active_model": "gpt-5-thinking",
  "active_reasoning": "high",
  "active_session_id": "2026-05-30T12-33-10Z-codex",
  "last_cwd": "/Users/dawid/project"
}
```

- All fields nullable; absent file ⇒ treated as all-null (fresh install).
- On read, cross-checked against config: an `active_agent`/`active_model` no longer present in config is
  treated as unset and repaired on next selection.
- Consumed by `shap status --json` for the prompt segment (read-only, cheap — Principle V / NFR-2).

## last-command-output.txt — `~/.local/share/shap/last-command-output.txt`

- Plain text: combined stdout+stderr of the most recent `:run`/pipe capture, truncated to
  `history.max_output_bytes` (truncation flagged in the `:read` payload).
- Metadata (command, exit code, captured_at) stored alongside (sibling JSON or a parsed header).
- Overwritten by each new capture (MVP keeps only the latest).

## Path resolution

| Purpose | Path | Override |
|---------|------|----------|
| Config | `${XDG_CONFIG_HOME:-~/.config}/shap/config.toml` | `--config <path>` / `SHAP_CONFIG` env |
| State | `${XDG_DATA_HOME:-~/.local/share}/shap/state.json` | `SHAP_DATA_DIR` env |
| Sessions | `<history.dir>` (default under data dir) | `[history].dir` |
| Capture | `${XDG_DATA_HOME:-~/.local/share}/shap/last-command-output.txt` | `SHAP_DATA_DIR` env |

`~` and `$XDG_*` are expanded; on macOS the XDG defaults above are used (not the Apple container paths)
for predictability and parity with Linux.
