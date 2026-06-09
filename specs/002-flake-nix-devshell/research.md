# Phase 0 Research: Nix Flake — Dev Shell, Package, and App

**Feature**: `002-flake-nix-devshell` | **Date**: 2026-06-03

Decisions resolving the Technical Context. No NEEDS CLARIFICATION remained; this records *why* each
choice was made and what was rejected.

## D1 — System enumeration: `lib.genAttrs`, not `flake-utils`

- **Decision**: Define `forAllSystems = nixpkgs.lib.genAttrs supportedSystems` where
  `supportedSystems = [ "aarch64-darwin" "x86_64-darwin" "aarch64-linux" "x86_64-linux" ]`. Build a
  per-system `pkgs` with the rust-overlay applied, then map `devShells`/`packages`/`apps` over it.
- **Rationale**: One fewer flake input (Constitution IX). The four MVP targets are a fixed, small set;
  a helper closure is clearer than pulling a framework. Invoking the flake on an unsupported system
  yields a missing-attribute error that names the system (FR-008).
- **Alternatives rejected**: `flake-utils.eachDefaultSystem` (adds an input and pulls in systems we do
  not support); `flake-parts` (heavier module system, unjustified for three outputs — Constitution I/II).

## D2 — Rust toolchain pin: `rust-toolchain.toml` via oxalica `fromRustupToolchainFile`

- **Decision**: Add `rust-toolchain.toml` with `channel = "1.88.0"` and components
  `["clippy" "rustfmt" "rust-src"]`. In the flake, after applying `rust-overlay.overlays.default`,
  resolve `rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml`. Use this one
  derivation in both the dev shell and the package build.
- **Rationale**: Single source of truth for the toolchain (FR-002, FR-013). The same file is what
  `rustup` and editors (rust-analyzer) read, so contributors inside and outside Nix agree on the
  version. oxalica's `fromRustupToolchainFile` is the documented path for this.
- **Alternatives rejected**: `rust-bin.stable."1.88.0".default` inline (duplicates the version in two
  places, drifts from any rustup users); nixpkgs' own `rustc`/`cargo` (not pinned to 1.85, may lag,
  no edition-2024 guarantee).

## D3 — Package build: `buildRustPackage` over `makeRustPlatform`-wrapped pinned toolchain

- **Decision**:
  ```nix
  rustPlatform = pkgs.makeRustPlatform { cargo = rustToolchain; rustc = rustToolchain; };
  shap = rustPlatform.buildRustPackage {
    pname = "shap";
    version = "0.1.0";              # tracks workspace.package.version
    src = ./.;                       # or a cleaned source
    cargoLock.lockFile = ./Cargo.lock;
    # builds the workspace; the `shap` binary (crates/shap-cli [[bin]] name = "shap") lands in $out/bin
  };
  ```
- **Rationale**: `makeRustPlatform` makes the package use the *same* pinned toolchain as the dev shell
  (FR-006, FR-013), not nixpkgs' default rustc. `cargoLock.lockFile = ./Cargo.lock` consumes the
  committed lock and errors if it is missing or inconsistent with `Cargo.toml` (FR-011) instead of
  re-resolving. The binary's name is `shap` (`crates/shap-cli` `[[bin]] name = "shap"`), so it lands at
  `$out/bin/shap` and becomes `packages.default` (FR-005, FR-010).
- **Alternatives rejected**: `crane` (more capable incremental-build framework, but an extra input and
  more concepts than this MVP needs — Constitution I/II); vendoring deps via `cargoHash`/`importCargoLock`
  by hand (more fragile than `cargoLock.lockFile`); using stock `pkgs.rustPlatform` (wrong toolchain
  version, violates FR-013).
- **Note**: `agent-client-protocol` and friends are normal crates.io deps already in `Cargo.lock`; no
  special handling. If any dep needs native libs at build time they go in `nativeBuildInputs`/`buildInputs`
  (e.g. `pkg-config`, `openssl`) — to be added only if the build reports a missing lib (YAGNI).

## D4 — App output

- **Decision**: `apps.default = { type = "app"; program = "${shap}/bin/shap"; }`.
- **Rationale**: `nix run` with no sub-command starts `shap` and shows usage/help (FR-007, FR-010,
  US3). Trivial and standard.
- **Alternatives rejected**: a wrapper script (unneeded indirection).

## D5 — ACP agents and coding agents from `numtide/llm-agents.nix`

- **Decision**: Add `llm-agents.url = "github:numtide/llm-agents.nix"`. In the **dev shell only**,
  include `llm-agents.packages.${system}.codex-acp` and `llm-agents.packages.${system}.claude-agent-acp`
  (optionally `claude-code`). These are the ACP adapters `shap` talks to.
- **Rationale**: Lets a contributor exercise the full `:`-command flow against a real ACP agent without
  hand-installing npm/curl-bash agents (FR-004, US1 acceptance 3 context). Using the flake's prebuilt
  `packages.${system}.*` hits the Numtide cache (fast, no local compile). They are dev-shell extras, so
  their presence never gates `nix build`/tests (FR-004).
- **Alternatives rejected**: `overlays.shared-nixpkgs` to rebuild agents against our `nixpkgs` (rebuilds
  from source, slow, no cache hit — only needed if we required a single nixpkgs for the agents, which we
  do not); bundling agents into the **package** output (agents are runtime peers a user supplies, not
  part of the `shap` binary — out of scope per spec assumptions).
- **Input cost note (Constitution IX)**: `llm-agents` does not `follows` our `nixpkgs` because its
  prebuilt packages are pinned to its own nixpkgs for cache hits; that is intentional and the reason we
  consume `packages.*` rather than its overlay.

## D6 — Dev-shell tool set

- **Decision**: `mkShell` with `packages = [ rustToolchain ] ++ [ cargo-nextest cargo-deny rust-analyzer ]
  ++ [ fzf git ] ++ [ codex-acp claude-agent-acp ]`.
- **Rationale**: `rustToolchain` covers `cargo`/`rustc`/`clippy`/`rustfmt`/`rust-src` (build, test,
  format, lint — FR-003). `cargo-nextest` is the project's test runner (per `001` plan). `cargo-deny`
  matches the committed `deny.toml`. `rust-analyzer` supports editor work. `fzf` + `git` are the
  optional runtime helpers `shap` uses (pickers, `:commit`) (FR-004). The ACP adapters come from D5.
- **Alternatives rejected**: adding `cargo-udeps` (needs nightly; defer — YAGNI); shipping a full TUI
  toolchain (unused).
- **`RUST_SRC_PATH`**: set from `rustToolchain` so rust-analyzer resolves std sources.

## D7 — `flake.lock` committed; `nix flake check` as the gate

- **Decision**: Commit `flake.lock` (FR-009). CI/contributors verify with `nix flake check` and a
  build smoke test (`nix build` then `result/bin/shap --version`).
- **Rationale**: The lock is what makes every consumer resolve identical inputs (SC-004). `flake check`
  evaluates all outputs and catches breakage.
- **Alternatives rejected**: leaving the lock ungenerated (non-reproducible — violates FR-006/FR-009).

## D8 — direnv (`.envrc`) — out of scope, noted

- **Observation**: The existing `.envrc` reads `use flake github:dgabka/config.nix#rust` (an external
  personal config), not this repo's flake.
- **Decision**: Do not modify `.envrc` as part of this feature. The quickstart documents the optional
  `use flake` line for contributors who want auto-entry into *this* flake's dev shell.
- **Rationale**: `.envrc` is user/environment-specific; changing it is outside the spec's developer-
  facing outputs (spec Assumptions). Documenting the option is enough.
