# Getting started

This walkthrough takes you from a fresh install to your first agent reply, then tours the rest of
the commands. It uses **Claude Code** (via its ACP adapter) as the example agent — but any
ACP-compatible agent works; see [agents.md](./agents.md).

## Prerequisite

`shap` installed and on your `PATH`. If not, see [Installation](./installation.md).

```sh
shap --version
```

## 1. Configure an agent

`shap` reads a single TOML file at `~/.config/shap/config.toml` (it never rewrites it). Create one
with a Claude Code agent:

```toml
default_agent = "claude"

[agents.claude]
command = "claude-agent-acp"          # an ACP adapter binary on your PATH
models = ["sonnet", "opus"]
default_model = "sonnet"
```

`command` is the external ACP adapter `shap` launches for each prompt; it must resolve on your
`PATH`. `models` is the exact list `:model` will offer. To add more agents (e.g. Codex) or carry
adapter-specific keys, see [agents.md](./agents.md). For every field and default, see
[config.md](./config.md).

## 2. Verify setup

```sh
shap doctor
```

This checks config validity, whether each agent's `command` is on `PATH`, picker/git presence,
session-dir writability, and shell-integration status — with a remediation line per failure. Fix any
`[FAIL]` lines before continuing.

## 3. Send your first prompt

```sh
shap send "hello"
```

The prompt goes to the active agent and its reply prints in the terminal. With the optional
[shell integration](./shell-integration.md) the same thing is one keystroke shorter — `: hello`
(colon + space) is exactly equivalent to `shap send "hello"`.

You can pull files into a prompt with `@file` references:

```sh
shap send "explain @src/main.rs"
```

## Command tour

Every command below is a plain `shap` subcommand. When the Zsh integration is installed, each also
has an equivalent `:` form (shown in the last column) — the two are interchangeable.

| Command | What it does | Example | Shell form |
|---------|--------------|---------|------------|
| `send <prompt>` | Send a prompt to the active agent (supports `@file`). | `shap send "write tests"` | `: write tests` |
| `agent [name]` | Select the active agent; no name opens a picker. | `shap agent claude` | `:agent [name]` |
| `model [name]` | Select a model from the active agent's list; no name opens a picker. | `shap model opus` | `:model [name]` |
| `reasoning [level]` | Set reasoning effort (`low`/`medium`/`high`); no level opens a picker. | `shap reasoning high` | `:reasoning` / `:effort` |
| `new` | Start a new session, keeping agent/model/reasoning. | `shap new` | `:new` |
| `status` | Show the active agent/model/reasoning/session (`--json` for scripts). | `shap status` | `:status` |
| `run -- <cmd…>` | Run a command and capture its combined output for later. | `shap run -- cargo test` | `:run cargo test` |
| `read [prompt]` | Send a prompt plus the last captured output (or piped stdin). | `shap read "why did this fail?"` | `:read why did this fail?` |
| `commit` | Generate a commit message and print a `git commit` line (never runs it). | `shap commit` | `:commit` |
| `doctor` | Validate the installation and configured agents. | `shap doctor` | `:doctor` |
| `config` | Inspect configuration; `--schema` prints the JSON schema. | `shap config --schema` | — |
| `completions <shell>` | Generate shell completions (`bash`/`zsh`/`fish`/`elvish`/`powershell`). | `shap completions zsh` | — |

Global options on every command: `--cwd <path>` (working directory; the shell forwards it) and
`--config <path>` (override the config file location).

A typical loop — run something, then ask the agent about the result:

```sh
shap run -- cargo test
shap read "summarize the failures and suggest a fix"
```

## Where to go next

- [Shell integration](./shell-integration.md) — install the Zsh `:` commands and prompt segment.
- [Agents](./agents.md) — configure multiple agents and adapter passthrough.
- [Configuration](./config.md) — all config fields, state, and defaults.
- [Documentation index](./index.md) — all guides.
