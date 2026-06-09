# Quickstart: shap Nix Flake

**Feature**: `002-flake-nix-devshell` | **Date**: 2026-06-03

How a contributor or user works with `shap` through the flake. Doubles as the manual acceptance
walkthrough for the spec's user stories.

## Prerequisites

- Nix with flakes enabled (`experimental-features = nix-command flakes`).
- Nothing else — the flake provides the Rust toolchain and tooling.

## US1 — Enter the dev shell

```sh
cd shap
nix develop
```

Inside the shell:

```sh
rustc --version                      # 1.88.0 (pinned)
cargo build --workspace
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets
codex-acp --help                     # optional ACP adapter, from llm-agents.nix
```

All tools come from the shell; nothing is installed on the host.

Optional direnv auto-entry — add to `.envrc` (do not commit over the existing personal line unless you
intend to):

```sh
use flake
```

## US2 — Build the package

```sh
nix build                            # default output == shap
./result/bin/shap --version
# explicit form:
nix build .#shap
```

`result/bin/shap` is a self-contained binary built with the pinned toolchain against the committed
`Cargo.lock`. On a clean machine with no host Rust toolchain, this still succeeds.

## US3 — Run the app without installing

```sh
nix run . -- --help                  # starts shap, prints usage
nix run . -- doctor                  # environment self-check
```

`nix run github:dgabka/shap` works once pushed (no clone needed).

## Verify the whole feature

```sh
nix flake check                      # evaluate all outputs
nix flake metadata                   # confirm locked inputs (nixpkgs, rust-overlay, llm-agents)
```

## Notes

- Supported systems: `aarch64-darwin`, `x86_64-darwin`, `aarch64-linux`, `x86_64-linux`. Other systems
  fail with a clear missing-attribute error.
- The Rust version is pinned once in `rust-toolchain.toml` and shared by the dev shell and the package
  build, so what you test is what you ship.
- ACP agents (`codex-acp`, `claude-agent-acp`) are dev-shell conveniences only; they are not bundled
  into the `shap` package.
