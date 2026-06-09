# Implementation Plan: Nix Flake — Dev Shell, Package, and App

**Branch**: `002-flake-nix-devshell` | **Date**: 2026-06-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-flake-nix-devshell/spec.md`

## Summary

Add a root `flake.nix` that gives `shap` three reproducible Nix outputs: a **dev shell** for
contributors (pinned Rust toolchain + build/test/lint tooling + optional ACP agents), a **package**
that builds the `shap` binary from the Cargo workspace, and an **app** that runs it directly. The
Rust toolchain is pinned once via a `rust-toolchain.toml` resolved through the **oxalica/rust-overlay**
and shared by both the dev shell and the package build (single source of truth). ACP agent adapters
and other coding agents come from **numtide/llm-agents.nix** and are surfaced only in the dev shell as
optional helpers. The package build consumes the committed `Cargo.lock` so dependency versions are
fully pinned and a stale lock fails loudly.

## Technical Context

**Language/Version**: Nix flakes (experimental features `nix-command flakes`); packaged target is the
existing Rust workspace pinned at Rust 1.88 / edition 2024.

**Primary Dependencies** (flake inputs):
- `nixpkgs` — `github:NixOS/nixpkgs/nixpkgs-unstable` (package set + `lib`, `rustPlatform`, `mkShell`).
- `rust-overlay` — `github:oxalica/rust-overlay` (`inputs.nixpkgs.follows = "nixpkgs"`). Provides
  `rust-bin` for an exact, reproducible Rust toolchain.
- `llm-agents` — `github:numtide/llm-agents.nix`. Provides ACP adapters (`codex-acp`,
  `claude-agent-acp`) and coding agents, daily-built and cached.

No `flake-utils`: systems are enumerated with `nixpkgs.lib.genAttrs` over the four MVP targets to keep
the input set minimal (Constitution IX).

**Storage**: N/A (build/dev tooling). Pinned input revisions recorded in committed `flake.lock`.

**Testing**: `nix flake check`; `nix develop -c cargo nextest run --workspace`; `nix build` produces a
runnable binary verified with `result/bin/shap --version` / `result/bin/shap doctor`; `nix run . -- --help`.

**Target Platform**: `aarch64-darwin`, `x86_64-darwin`, `aarch64-linux`, `x86_64-linux` (mirrors the
`001` plan; Windows out of scope).

**Project Type**: Build/packaging infrastructure for an existing single Cargo workspace.

**Performance Goals**: Cached builds resolve from the Numtide/Nix caches without recompiling agents;
first-run dev-shell entry is dominated by toolchain/agent substitution, not local compilation.

**Constraints**: Reproducible (all inputs pinned, no host toolchain reliance); the dev shell and the
package build MUST use the *same* pinned toolchain; the package build MUST reuse the committed
`Cargo.lock` and fail on mismatch; optional helpers MUST NOT block build/test when unavailable.

**Scale/Scope**: One `flake.nix`, one `rust-toolchain.toml`, one committed `flake.lock`, and a docs
page. Three outputs (`devShells.default`, `packages.default`, `apps.default`) across four systems.

*No NEEDS CLARIFICATION remain — the two external inputs and the pinning strategy were specified in the
planning input and confirmed against upstream. Phase 0 records the decisions.*

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Evaluated against Constitution v1.0.0.

| # | Principle | Status | Notes |
|---|-----------|--------|-------|
| I | Keep It Simple (KISS) | ✅ Pass | Plain `genAttrs` over four systems instead of a flake-parts/flake-utils framework; stock `buildRustPackage`; one toolchain definition reused. |
| II | Keep It Lean (YAGNI) | ✅ Pass | Only the three requested outputs. No CI output, no cross-compilation matrix, no extra apps beyond `shap`. |
| III | Code Quality | ✅ Pass | Flake reads top-down; a `forAllSystems` helper and a single `mkPackage`/`mkDevShell` per system keep it idiomatic. |
| IV | Tests for Meaningful Logic | ✅ Pass | `nix flake check` + a build smoke (`result/bin/shap --version`) cover the meaningful behavior; no logic to unit-test. |
| V | Readability Over Performance Tricks | ✅ Pass | No build hacks; rely on the binary cache for speed. |
| VI | Fail Clearly | ✅ Pass | `cargoLock.lockFile` surfaces lock mismatches explicitly; unsupported systems error with a missing-attr message naming the system. |
| VII | Keep User Control | ✅ Pass | Build/packaging only; touches no repos and runs nothing destructive. |
| VIII | Respect the Shell | ✅ Pass | No change to the interactive shell path; `nix develop` is opt-in and separate from `shap`'s Zsh integration. |
| IX | Minimize Dependencies | ✅ Pass | Three flake inputs, each justified; `rust-overlay` follows `nixpkgs`; no `flake-utils`. |
| X | Preserve Contributor Clarity | ✅ Pass | A single root `flake.nix` + `rust-toolchain.toml`; docs page explains the three commands. |

**Gate result**: PASS — no violations, Complexity Tracking not required. Re-evaluated after Phase 1:
still PASS (design added no inputs or indirection beyond what is documented here).

## Project Structure

### Documentation (this feature)

```text
specs/002-flake-nix-devshell/
├── plan.md              # This file
├── research.md          # Phase 0 output (decisions + rationale)
├── data-model.md        # Phase 1 output (flake output entities)
├── quickstart.md        # Phase 1 output (develop/build/run)
├── contracts/
│   └── flake-outputs.md # Phase 1 output (flake output contract)
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

```text
shap/
├── flake.nix              # NEW — inputs (nixpkgs, rust-overlay, llm-agents); outputs devShells/packages/apps
├── flake.lock             # NEW — pinned input revisions (committed)
├── rust-toolchain.toml    # NEW — single toolchain pin (channel "1.88.0" + clippy/rustfmt/rust-src)
├── Cargo.toml             # existing workspace manifest (unchanged)
├── Cargo.lock             # existing — consumed by the package build (unchanged)
├── crates/                # existing workspace crates (unchanged)
├── docs/
│   └── nix.md             # NEW — how to use the flake (develop/build/run)
└── .envrc                 # existing — optional: point at this flake's devShell (see research.md)
```

**Structure Decision**: A single root `flake.nix` is the entry point (FR-001). The toolchain is pinned
in `rust-toolchain.toml` and resolved with `rust-bin.fromRustupToolchainFile`, then reused by both the
dev shell and the package's `makeRustPlatform` so the test environment matches the build environment
(FR-013). The package uses `buildRustPackage { cargoLock.lockFile = ./Cargo.lock; }` to consume the
committed lock and fail on mismatch (FR-011). No new Rust code or crate changes — this feature is
purely additive packaging at the repo root.

## Complexity Tracking

> No Constitution violations — section intentionally empty.
