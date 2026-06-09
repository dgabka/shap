# Phase 1 Data Model: Project Documentation

The "data model" for a documentation feature is the document inventory: each artifact, its purpose,
the spec requirements it satisfies, its required sections, and its links to other documents. This is
the authoritative map Phase 2 tasks build against.

## Entities (documents)

### README.md (front page) — NEW

- **Purpose**: First-screen identity + value proposition + navigation. Overview-scoped.
- **Satisfies**: FR-001, FR-002, FR-002a, FR-014; SC-001.
- **Required sections**:
  - Title + one-line description (what `shap` is)
  - "What it does" — core capabilities bullet list (shell-native ACP agent chat, agent/model/
    reasoning switching, `:run`/`:read` context, `:commit` helper, `doctor` self-check)
  - Quick install (shortest path; links to full installation guide)
  - Minimal usage example (configure one agent → `: hello` / `shap send`)
  - Links section → `docs/index.md`, installation, getting-started
  - License + maturity one-liner
- **Constraints**: No full command reference inline (FR-002a). All deep content via links.
- **Links to**: `docs/index.md`, `docs/installation.md`, `docs/getting-started.md`.

### docs/index.md (documentation index) — NEW

- **Purpose**: Catalog of all topic guides with one-line descriptions.
- **Satisfies**: FR-009, FR-014; SC-005.
- **Required sections**: one entry per guide (getting-started, installation, agents, config,
  shell-integration, nix), each `[title](path) — one-line description`.
- **Constraints**: Every entry links to an existing file; no orphan guides, no broken links.
- **Links to**: every other doc.

### docs/installation.md — NEW

- **Purpose**: Prerequisites + the two in-scope install methods + verification.
- **Satisfies**: FR-003, FR-011; SC-002.
- **Required sections**:
  - Prerequisites (Rust toolchain per `rust-toolchain.toml` / `rust-version = 1.85`; or Nix with
    flakes enabled)
  - Method A — build from source with cargo (`cargo build --release`, resulting binary path, adding
    to PATH)
  - Method B — Nix flake (`nix run`, `nix profile install`, devshell via `nix develop` / direnv)
  - Trade-offs between A and B
  - Verify install (`shap --version` / `shap doctor`)
  - Troubleshooting pointer (`shap doctor`, common PATH issue)
- **Constraints**: Prebuilt binaries NOT documented. Commands match `flake.nix` outputs.
- **Links to**: `nix.md` (deep flake details), `getting-started.md`, back to `index.md`.

### docs/getting-started.md — NEW

- **Purpose**: Fresh install → first-run setup → first successful task, end to end.
- **Satisfies**: FR-004, FR-005, FR-006, FR-008; SC-003, SC-004.
- **Required sections**:
  - Prerequisite: installed `shap` (link to installation)
  - First-run config: create `config.toml` with a Claude Code agent block
    (`[agents.claude]`, `command = "claude-agent-acp"`, models `sonnet`/`opus`) — note any ACP agent
    works (link to `agents.md`)
  - Verify with `shap doctor`
  - First chat: `: hello` (shell) ≡ `shap send "hello"` (CLI) — equivalence noted (FR-008)
  - Core commands tour: `:agent`/`:model`/`:reasoning`, `:new`, `:status`, `:run`, `:read`,
    `:commit`, each with purpose + one example, each shown with its `shap <subcommand>` equivalent
  - Where to go next (links to config, agents, shell-integration)
- **Constraints**: Claude Code is the worked example. Every command verified against the CLI.
- **Links to**: `installation.md`, `agents.md`, `config.md`, `shell-integration.md`.

### docs/agents.md — EXISTING (review/align)

- **Purpose**: Agent configuration & ACP model.
- **Satisfies**: FR-005 (agent commands), FR-006 (agent config).
- **Action**: Verify commands/fields still current; ensure it is linked from `index.md` and
  cross-linked from getting-started. Add a one-line description for the index.

### docs/config.md — EXISTING (review/align)

- **Purpose**: Config + state file reference (paths, fields, defaults).
- **Satisfies**: FR-006; supports SC-007.
- **Action**: Verify field/default accuracy against current config schema; index entry; cross-link.

### docs/shell-integration.md — EXISTING (review/align)

- **Purpose**: Zsh `:`-command mapping and install of the shell layer.
- **Satisfies**: FR-007, FR-008; supports US2 integration scenario.
- **Action**: Confirm the `:`↔`shap` mapping table is current and authoritative (reused, not
  duplicated, by getting-started); index entry; ensure the "usable without the shell layer"
  statement is present (FR-007).

### docs/nix.md — EXISTING (review/align)

- **Purpose**: Flake devshell/package/app deep reference.
- **Satisfies**: FR-003 (Nix method depth).
- **Action**: Confirm flake commands match `flake.nix`; ensure `installation.md` links here for
  depth; index entry.

## Relationships (link graph)

```text
README.md ──► docs/index.md ──► { getting-started, installation, agents, config, shell-integration, nix }
README.md ──► docs/installation.md ──► docs/nix.md
README.md ──► docs/getting-started.md ──► { installation, agents, config, shell-integration }
docs/installation.md ──► docs/getting-started.md
```

Invariant (SC-005): the link graph is fully connected from `README.md`; every `index.md` entry
resolves to an existing file; no document is orphaned.

## Validation rules (apply before publication)

- **R1 — Command accuracy**: every command/flag shown exists in `shap --help` / `shap <cmd> --help`
  or `shell/zsh/shap.zsh` (FR-013, SC-007).
- **R2 — Link integrity**: every relative link resolves to an existing file/anchor (SC-005).
- **R3 — Coverage**: every user-facing command (`send`, `agent`, `model`, `reasoning`/`effort`,
  `new`, `status`, `run`, `read`, `doctor`) is documented with purpose + ≥1 example (FR-005, SC-004).
- **R4 — Install scope**: only cargo + Nix install paths appear; no prebuilt-binary instructions
  (FR-003).
- **R5 — README scope**: README carries no full command reference; depth is reached via links
  (FR-002a).
- **R6 — Equivalence**: dual-surface commands show both `:` and `shap` forms, labeled equivalent
  (FR-008).
- **R7 — Metadata**: license, platforms, maturity, issue channel present (FR-012, SC-006).
