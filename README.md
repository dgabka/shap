# shap

**[Sh]ell [A]gent [P]roxy**

**A shell-native interface for ACP coding agents.** Chat with coding agents (Claude Code, Codex, and
other [ACP](https://agentclientprotocol.com)-compatible agents) straight from your terminal — no
editor, no web UI. Type a colon and a prompt; the agent's reply prints inline.

## What it does

- **Chat from the shell** — `: <prompt>` (or `shap send`) sends a prompt to the active agent and
  streams the reply in your terminal.
- **Switch agents, models, and reasoning** on the fly, with interactive pickers when you omit a name.
- **Feed context to the agent** — capture a command's output with `:run` and ask about it with
  `:read`, or pull files into a prompt with `@file`.
- **Commit helper** — `:commit` generates a `git commit` line and loads it into your buffer for
  review (it never runs the commit).
- **Self-check** — `shap doctor` validates your install and configured agents with actionable fixes.
- **Thin, optional shell layer** — every `:` command maps to a plain `shap <subcommand>`, so the
  tool is fully usable without the shell integration.

## Quick start

```sh
# Install (from source; or use the Nix flake — see the installation guide)
cargo install --path crates/shap-cli

# Configure one agent at ~/.config/shap/config.toml
#   default_agent = "claude"
#   [agents.claude]
#   command = "claude-agent-acp"
#   models = ["sonnet", "opus"]
#   default_model = "sonnet"

# Send your first prompt
shap send "hello"
```

Full steps in [Installation](./docs/installation.md) and [Getting started](./docs/getting-started.md).

## Documentation

- [Documentation index](./docs/index.md) — all guides
- [Installation](./docs/installation.md) — cargo or Nix
- [Getting started](./docs/getting-started.md) — first agent reply + command tour
- [Agents](./docs/agents.md) · [Configuration](./docs/config.md) ·
  [Shell integration](./docs/shell-integration.md) · [Nix flake](./docs/nix.md)

## Status

Pre-1.0 (`0.1.0`) and under active development — interfaces may change between releases. Runs on
macOS and Linux; the shell integration targets Zsh.

## Contributing

Issues and pull requests are welcome at <https://github.com/dgabka/shap>.

## License

Licensed under the [Apache License 2.0](./LICENSE).
