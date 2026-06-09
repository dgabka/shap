# Phase 0 Research: Project Documentation

All Technical Context items are resolved; there are no outstanding NEEDS CLARIFICATION markers
(the three spec clarifications already fixed install scope, README depth, and example agent). The
decisions below record the documentation-design choices that drive Phase 1.

## Decision 1 — Documentation home & format

- **Decision**: Root `README.md` as the front page plus a flat `docs/` directory of Markdown topic
  guides. No static-site generator, no nested doc folders.
- **Rationale**: Matches the project's existing layout (four guides already live in `docs/`). Plain
  Markdown renders on the repository host with zero tooling, honoring KISS and Minimize Dependencies.
- **Alternatives considered**: mdBook / Docusaurus hosted site (rejected — YAGNI for a pre-1.0 CLI;
  adds build + deploy surface); single mega-README (rejected — drift magnet, contradicts the
  overview+links clarification).

## Decision 2 — README scope (overview + links)

- **Decision**: README contains identity, a short capability list, a quick-install snippet, one
  minimal usage example, then a links section into `docs/`. No full command reference inline.
- **Rationale**: Per spec clarification (FR-002a). The README is the highest-traffic, highest-drift
  surface; keeping reference detail in `docs/` gives one source of truth per topic.
- **Alternatives considered**: full self-contained README (rejected by clarification — duplication
  and drift); badge-only minimal README (rejected — fails SC-001 "know what it does from the front
  page").

## Decision 3 — Installation methods in scope

- **Decision**: Document two methods — (a) build from source with cargo, (b) install/run via the
  Nix flake. Prebuilt release binaries (produced by `.github/workflows/release.yml` for
  macOS/Linux × arm/x86) are explicitly out of scope this iteration.
- **Rationale**: Spec clarification. Cargo and Nix are the reproducible, already-documented-ish
  paths; the release pipeline's artifact naming/versioning is better documented once it stabilizes.
- **Alternatives considered**: include prebuilt binaries (deferred — avoids documenting an
  install path whose download URLs/checksums aren't yet settled); Nix-only (rejected — excludes
  non-Nix users).
- **Source of truth**: `flake.nix` outputs (`packages.default = shap`, `apps.default`, devshell),
  `rust-toolchain.toml` (pinned toolchain → cargo prerequisite), `Cargo.toml` (`rust-version =
  1.85`, edition 2024).

## Decision 4 — Getting-started example agent

- **Decision**: Walkthrough uses Claude Code via an ACP adapter as the worked example, with a note
  that any ACP-compatible agent works. Config example mirrors `docs/config.md`'s `[agents.claude]`
  block (`command = "claude-agent-acp"`, models `sonnet`/`opus`).
- **Rationale**: Spec clarification (Q3). Most recognizable agent; concrete config lowers
  first-run friction. The "any ACP agent" note preserves generality and matches `docs/agents.md`.
- **Alternatives considered**: Codex example (valid but less universally recognized); fully
  agent-agnostic placeholder (rejected — a concrete copy-pasteable example tests better against
  SC-003 first-attempt success).

## Decision 5 — Command reference accuracy strategy

- **Decision**: Hand-write the command reference in `docs/` (likely folded into getting-started +
  the existing guides), and gate publication on a manual cross-check: each documented command/flag
  is confirmed against `shap --help` / `shap <cmd> --help` and `shell/zsh/shap.zsh`; each
  intra-doc link is confirmed to resolve.
- **Rationale**: The CLI surface is small and stable; generating reference docs from `--help` would
  add tooling for little gain (YAGNI). A manual accuracy gate satisfies FR-013/SC-007 without a
  build step.
- **Alternatives considered**: auto-generate from clap (rejected now — tooling cost; revisit if the
  command surface grows). Canonical command list to verify against: `send`, `agent`, `model`,
  `reasoning` (alias `effort`), `new`, `status`, `run`, `read`, `doctor`, plus completions.

## Decision 6 — Command/shell equivalence presentation

- **Decision**: Where a `:` shell command maps to a `shap <subcommand>` (e.g., `:agent` ↔
  `shap agent`), document both forms in a single table and label them equivalent (FR-008). The
  authoritative mapping table already exists in `docs/shell-integration.md` and is reused, not
  duplicated.
- **Rationale**: Avoids divergence between the two surfaces; one mapping table, linked from
  getting-started rather than copied.
- **Alternatives considered**: document only `shap <subcommand>` (rejected — hides the headline
  `:`-command UX); document only `:` forms (rejected — FR-007 says the tool is usable without the
  shell layer).

## Decision 7 — Maturity, license, platforms, contribution

- **Decision**: State Apache-2.0 license (from `Cargo.toml`), supported platforms (macOS + Linux;
  zsh for the shell layer), pre-1.0 maturity with a stability caveat, and point issues/contributions
  at the GitHub repository (`https://github.com/dgabka/shap`).
- **Rationale**: FR-012 / SC-006. Honest pre-1.0 framing aligns with Preserve Contributor Clarity.
- **Alternatives considered**: omit maturity caveat (rejected — sets false stability expectations).
