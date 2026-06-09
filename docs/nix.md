# Nix flake

`shap` ships a flake at the repository root. It gives you a reproducible developer environment, a
buildable package, and a runnable app — all using a single pinned Rust toolchain.

## Prerequisites

- Nix with flakes enabled (`experimental-features = nix-command flakes`).

Nothing else: the flake provides the Rust toolchain and tooling.

## Developer environment

```sh
nix develop
```

Inside the shell you get the pinned toolchain (`rustc`/`cargo`/`clippy`/`rustfmt`/`rust-src`) plus
`cargo-nextest`, `cargo-deny`, `rust-analyzer`, and the optional helpers `fzf` and `git`. Two ACP
adapters from [numtide/llm-agents.nix](https://github.com/numtide/llm-agents.nix) are also on PATH —
`codex-acp` and `claude-agent-acp` — so you can exercise the full command flow.

```sh
cargo build --workspace
cargo nextest run --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets
```

`RUST_SRC_PATH` is set so rust-analyzer resolves the standard library.

Optional direnv auto-entry — add `use flake` to `.envrc` (the repo's existing `.envrc` points at a
personal config flake; do not overwrite it unless you mean to).

## Build the package

```sh
nix build            # default output == shap
./result/bin/shap --version
nix build .#shap     # explicit
```

The binary is built with the same pinned toolchain as the dev shell and against the committed
`Cargo.lock`, so a stale lock fails the build instead of silently re-resolving.

## Run without installing

```sh
nix run . -- --help
nix run . -- doctor
nix run github:dgabka/shap   # once pushed, no clone needed
```

## Toolchain pin

The Rust version lives in `rust-toolchain.toml` (`channel = "1.88.0"` — the minimum the locked
dependencies require; the workspace `rust-version` field is an older, looser bound). It is resolved through the
[oxalica/rust-overlay](https://github.com/oxalica/rust-overlay) and shared by the dev shell and the
package build. Bump the version there and both follow.

## Supported systems

`aarch64-darwin`, `x86_64-darwin`, `aarch64-linux`, `x86_64-linux`. Other systems fail evaluation with
a message naming the missing system. Windows is out of scope.

## Verify the flake

```sh
nix flake check
nix flake metadata   # lists locked inputs: nixpkgs, rust-overlay, llm-agents
```

## See also

- [Installation](./installation.md) · [Getting started](./getting-started.md) ·
  [Documentation index](./index.md)
