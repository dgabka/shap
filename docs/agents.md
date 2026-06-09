# Configuring agents

`shap` talks to external **ACP** (Agent Client Protocol) agents. Each `shap send` launches the
configured agent as a child process, speaks ACP over its stdio for one prompt, and exits — so the
agent adapter must be an ACP-compatible binary on your PATH.

## Defining an agent

```toml
default_agent = "codex"

[agents.codex]
command = "codex-acp"            # the ACP adapter binary (+ optional flags)
models = ["gpt-5", "gpt-5-thinking"]
default_model = "gpt-5-thinking"
```

- `command` is split with shell-style word rules, so flags are allowed:
  `command = "codex-acp --acp --quiet"`.
- The binary must resolve on PATH. Verify with `shap doctor`.
- `models` defines the *only* models selectable for this agent (`:model` offers exactly this list).

Add as many agents as you like; switch between them with `:agent`.

## Agent-specific passthrough

Any keys under `[agents.<name>]` beyond `command` / `models` / `default_model` are preserved verbatim
and are **not** interpreted by `shap`. Use them to carry adapter-specific settings:

```toml
[agents.codex]
command = "codex-acp"
models = ["gpt-5"]
default_model = "gpt-5"
api_key_env = "OPENAI_API_KEY"   # opaque to shap; available to the adapter
workspace_trust = "full"
```

## Selection and the prompt segment

- `:agent [name]` — switch agent. If the current model is invalid for the new agent, it resets to that
  agent's `default_model`.
- `:model [name]` — pick from the active agent's models only.
- `:reasoning [level]` / `:effort [level]` — `low` | `medium` | `high`.
- With no argument, each opens a picker (`fzf` → `skim` → built-in fallback).

Selections persist across shells in `state.json` and appear in the prompt segment.

## Diagnostics

```sh
shap doctor
```

Reports config validity, whether each agent's command is on PATH, picker/git presence, session-dir
writability, and shell-integration status — with a remediation line per failure. A missing agent
command is a critical failure (exit 1); when an agent becomes unavailable mid-request, the error is
recorded in the session log and surfaced clearly.

## Notes / current limitations (MVP)

- Each prompt is a fresh ACP session; multi-turn context replay (resume) is not yet wired, so
  conversation continuity is at the local session-log level.
- Model/reasoning selection is tracked and shown, and is available to adapters via passthrough;
  forwarding it into the ACP session itself is adapter-dependent.

## See also

- [Getting started](./getting-started.md) · [Configuration](./config.md) ·
  [Documentation index](./index.md)
