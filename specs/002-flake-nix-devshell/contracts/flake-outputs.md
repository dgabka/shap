# Contract: Flake Outputs

**Feature**: `002-flake-nix-devshell` | **Date**: 2026-06-03

The interface this feature exposes is the set of flake outputs. This is the contract consumers and CI
depend on. Each row is independently verifiable.

## Supported systems

`aarch64-darwin`, `x86_64-darwin`, `aarch64-linux`, `x86_64-linux`. Every output below exists for each.
Any other system MUST fail evaluation with a message naming the missing system. (FR-008)

## Output: `devShells.<system>.default`

| Property | Contract |
|----------|----------|
| Entry | `nix develop` enters the shell. |
| Toolchain | Pinned Rust 1.88.0 with `cargo`, `rustc`, `clippy`, `rustfmt`, `rust-src` on PATH. (FR-002) |
| Build/test/lint | `cargo build --workspace`, `cargo nextest run --workspace`, `cargo fmt --all`, `cargo clippy --workspace` all succeed using only shell-provided tools. (FR-003) |
| Optional helpers | `fzf`, `git`, `codex-acp`, `claude-agent-acp` present; their absence on a host MUST NOT affect build/test. (FR-004) |

**Verify**:
```sh
nix develop -c rustc --version          # → 1.88.0
nix develop -c cargo nextest run --workspace
nix develop -c cargo clippy --workspace --all-targets
nix develop -c cargo fmt --all --check
nix develop -c codex-acp --help         # optional helper present
```

## Output: `packages.<system>.shap` (and `packages.<system>.default`)

| Property | Contract |
|----------|----------|
| Result | `nix build` yields `result/bin/shap`, a runnable executable. (FR-005, FR-010) |
| Toolchain | Built with the same pinned 1.88.0 toolchain as the dev shell. (FR-006, FR-013) |
| Lock | Uses committed `Cargo.lock`; a missing/inconsistent lock fails the build with a lock error, never a silent re-resolve. (FR-011) |
| Reproducible | No reliance on host-installed Rust; same revision → equivalent result on a given system. (FR-006, SC-004) |

**Verify**:
```sh
nix build .#shap
./result/bin/shap --version             # prints shap version
nix build                                # default == shap
nix flake check                          # evaluates all outputs
```

## Output: `apps.<system>.shap` (and `apps.<system>.default`)

| Property | Contract |
|----------|----------|
| Run | `nix run` (no sub-command) starts `shap` and prints usage/help without error. (FR-007, FR-010, US3) |
| Self-check | `nix run . -- doctor` executes the self-check. |

**Verify**:
```sh
nix run . -- --help
nix run . -- doctor
```

## Output: locked inputs

| Property | Contract |
|----------|----------|
| `flake.lock` | Committed; pins `nixpkgs`, `rust-overlay`, `llm-agents`. (FR-009) |

**Verify**:
```sh
nix flake metadata                       # lists locked revisions
git ls-files flake.lock                  # tracked
```

## Non-goals (explicit)

- No `checks` beyond `nix flake check`'s default evaluation; no CI workflow output.
- No cross-compilation; only native builds per system.
- Agents are dev-shell extras, never part of `packages.shap`.
