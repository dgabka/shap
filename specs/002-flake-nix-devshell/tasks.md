---
description: "Task list for Nix Flake — Dev Shell, Package, and App"
---

# Tasks: Nix Flake — Dev Shell, Package, and App

**Input**: Design documents from `/specs/002-flake-nix-devshell/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/flake-outputs.md, quickstart.md

**Tests**: No automated unit tests requested. Verification is via `nix` commands (`nix flake check`,
`nix develop -c …`, `nix build`, `nix run`) per the contract; each story phase ends with a verify task.

**Organization**: Tasks grouped by user story (US1 dev shell, US2 package, US3 app).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- All flake outputs live in a single `flake.nix`; tasks that edit it are therefore **sequential** (not
  `[P]`) even when conceptually independent. Only separate files (`rust-toolchain.toml`, `docs/nix.md`)
  are parallelizable.

## Path Conventions

Repository root (`/Users/dgabka/repos/shap/`): `flake.nix`, `flake.lock`, `rust-toolchain.toml`,
`docs/nix.md`. Existing `Cargo.toml`/`Cargo.lock`/`crates/` are consumed unchanged.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Pin the toolchain that every output reuses.

- [X] T001 [P] Create `rust-toolchain.toml` at repo root with `[toolchain]` `channel = "1.88.0"` and `components = ["clippy", "rustfmt", "rust-src"]` (single toolchain source of truth, FR-002/FR-013; see research.md D2). **Note**: pinned 1.88.0, not 1.85 — the locked deps (`darling 0.23`, `serde_with 3.20`, `time 0.3.47`) require rustc ≥ 1.88; the workspace `rust-version = "1.85"` is a stale lower bound.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Flake skeleton — inputs, per-system package set, and the shared `rustToolchain`. Every
user story output (devShell/package/app) is added to this same file, so it MUST exist first.

**⚠️ CRITICAL**: No user story output can be added until this phase is complete.

- [X] T002 Create `flake.nix` at repo root with `description` and `inputs`: `nixpkgs` = `github:NixOS/nixpkgs/nixpkgs-unstable`; `rust-overlay` = `github:oxalica/rust-overlay` with `inputs.nixpkgs.follows = "nixpkgs"`; `llm-agents` = `github:numtide/llm-agents.nix` (no follow — research.md D5) (FR-001)
- [X] T003 In `flake.nix` add `supportedSystems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ]` and `forAllSystems = nixpkgs.lib.genAttrs supportedSystems`; build per-system `pkgs` with `overlays = [ rust-overlay.overlays.default ]` and `rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml` (FR-008/FR-013; research.md D1/D2)
- [X] T004 Generate and commit `flake.lock` via `nix flake lock` (pins nixpkgs, rust-overlay, llm-agents — FR-009; research.md D7)

**Checkpoint**: `nix flake metadata` lists three locked inputs; flake evaluates with empty outputs.

---

## Phase 3: User Story 1 - Reproducible developer environment (Priority: P1) 🎯 MVP

**Goal**: `nix develop` yields the pinned toolchain + build/test/lint tooling + optional ACP agents.

**Independent Test**: On a clean machine with only Nix, `nix develop` then run build/test/fmt/clippy
successfully using only shell-provided tools.

### Implementation for User Story 1

- [X] T005 [US1] In `flake.nix` add `devShells = forAllSystems (system: { default = pkgs.mkShell { … }; })` wiring (FR-001)
- [X] T006 [US1] Populate the dev shell `packages` with `rustToolchain`, `cargo-nextest`, `cargo-deny`, `rust-analyzer`, `fzf`, `git`, and set `env.RUST_SRC_PATH` from `rustToolchain` (FR-002/FR-003/FR-004; research.md D6)
- [X] T007 [US1] Add ACP adapters to the dev shell from `llm-agents.packages.${system}.codex-acp` and `…claude-agent-acp` (optional helpers, MUST NOT gate build/test — FR-004; research.md D5)
- [X] T008 [US1] Verify per contracts/flake-outputs.md: `nix develop -c rustc --version` → 1.88.0; `nix develop -c cargo nextest run --workspace`; `nix develop -c cargo clippy --workspace --all-targets`; `nix develop -c cargo fmt --all --check`; `nix develop -c codex-acp --help` (SC-001/SC-002)

**Checkpoint**: US1 fully functional and independently testable (MVP).

---

## Phase 4: User Story 2 - Buildable package artifact (Priority: P2)

**Goal**: `nix build` produces a reproducible `shap` binary using the same pinned toolchain and the
committed `Cargo.lock`.

**Independent Test**: On a clean machine with no host Rust, `nix build` succeeds and `result/bin/shap`
runs.

### Implementation for User Story 2

- [X] T009 [US2] In `flake.nix` add `rustPlatform = pkgs.makeRustPlatform { cargo = rustToolchain; rustc = rustToolchain; }` and `packages = forAllSystems (system: rec { shap = rustPlatform.buildRustPackage { pname = "shap"; version = "0.1.0"; src = ./.; cargoLock.lockFile = ./Cargo.lock; }; default = shap; })` (FR-005/FR-006/FR-010/FR-011/FR-013; research.md D3)
- [X] T010 [US2] Verify per contracts/flake-outputs.md: `nix build .#shap` and `nix build` both succeed; `./result/bin/shap --version` → `shap 0.1.0`. **Note**: added `nativeCheckInputs = [ pkgs.git ]` because the build-time `cargo test` (commit tests) shells out to `git` (research.md D3 note).

**Checkpoint**: US1 and US2 both work independently.

---

## Phase 5: User Story 3 - Run the app without installing (Priority: P3)

**Goal**: `nix run` launches `shap` directly.

**Independent Test**: `nix run . -- --help` starts `shap` and prints usage; `nix run . -- doctor` runs.

### Implementation for User Story 3

- [X] T011 [US3] In `flake.nix` add `apps = forAllSystems (system: rec { shap = { type = "app"; program = "${self.packages.${system}.shap}/bin/shap"; }; default = shap; })` (FR-007/FR-010; research.md D4)
- [X] T012 [US3] Verify per contracts/flake-outputs.md: `nix run . -- --version` → `shap 0.1.0` (app launches) (SC-005)

**Checkpoint**: All three outputs independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [X] T013 [P] Write `docs/nix.md` covering `nix develop` / `nix build` / `nix run` and supported systems (FR-012; mirror quickstart.md)
- [X] T014 [P] No `docs/` index exists, so no cross-link added; the optional `use flake` direnv note lives in `docs/nix.md`. Existing personal `.envrc` left untouched (research.md D8). Also added `/result`, `/result-*` to `.gitignore`.
- [X] T015 Verified: flake evaluates across all 4 systems (`nix flake show`); package builds + tests pass; binary and `nix run` print `shap 0.1.0`; dev shell provides rustc/cargo 1.88.0, agents, fzf, git, rust-analyzer, RUST_SRC_PATH (SC-001..SC-006). `flake.lock` pins nixpkgs/rust-overlay/llm-agents.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 has no dependencies — start immediately.
- **Foundational (Phase 2)**: T002→T003 sequential (same file); T004 after T003. BLOCKS all stories.
- **User Stories (Phase 3–5)**: each depends on Phase 2. Because all three edit `flake.nix`, run them
  sequentially (P1→P2→P3) rather than in parallel; US3 (T011) additionally needs the package attr from
  US2 (T009) since the app points at `packages.${system}.shap`.
- **Polish (Phase 6)**: after the stories you intend to ship.

### User Story Dependencies

- **US1 (P1)**: after Phase 2. No dependency on US2/US3.
- **US2 (P2)**: after Phase 2. Independent of US1.
- **US3 (P3)**: after Phase 2; **depends on US2's `packages.shap`** (the app references it).

### Within Each User Story

- Add the output wiring before populating it; finish with the verify task.

### Parallel Opportunities

- T001 (`rust-toolchain.toml`) is [P] vs nothing else early.
- T013 and T014 (docs, separate files) are [P] with each other.
- Story-implementation tasks are **not** [P]: they share `flake.nix`.

---

## Parallel Example: Polish

```bash
# Different files, no dependencies:
Task: "Write docs/nix.md (T013)"
Task: "Cross-link Nix docs / note optional direnv use flake (T014)"
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 (T001) → Phase 2 (T002–T004) → Phase 3 (T005–T008).
2. **STOP and VALIDATE**: `nix develop` gives a working, pinned dev environment — ship as MVP.

### Incremental Delivery

1. Setup + Foundational → flake skeleton with locked inputs.
2. + US1 → reproducible dev shell (MVP).
3. + US2 → `nix build` package.
4. + US3 → `nix run` app.
5. Polish → docs + full quickstart validation.

---

## Notes

- All outputs share one `flake.nix`; respect the sequential ordering even where stories are logically
  independent, to avoid edit conflicts.
- The toolchain is pinned once (`rust-toolchain.toml`) and reused by both dev shell and package — keep
  them referencing the same `rustToolchain` (FR-013).
- Commit `flake.lock`; never leave it ungenerated.
- Agents are dev-shell extras only — do not add them to the `packages.shap` build.
- Total: 15 tasks (T001–T015).
