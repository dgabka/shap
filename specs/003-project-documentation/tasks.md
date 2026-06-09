---

description: "Task list for Project Documentation"
---

# Tasks: Project Documentation

**Input**: Design documents from `/specs/003-project-documentation/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/docs-structure.md, quickstart.md

**Tests**: Not applicable — this is a documentation feature. "Tests" here means the accuracy/link
verification gate (Polish phase), not an automated test suite.

**Organization**: Tasks are grouped by user story (spec priorities P1–P3) so each story is an
independently deliverable documentation increment.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: Which user story the task serves (US1–US5)
- Every task names an exact file path

## Path Conventions

Documentation lives at the repository root (`README.md`) and under `docs/`. Source of truth for
accuracy: `crates/` (CLI), `shell/zsh/shap.zsh` (`:` commands), `flake.nix` (Nix install),
`Cargo.toml` (license/version). Source files are NOT modified by this feature.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Establish the documentation baseline.

- [X] T001 Confirm the existing doc layout and that the four guides are present and readable: `docs/agents.md`, `docs/config.md`, `docs/shell-integration.md`, `docs/nix.md`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Capture the authoritative facts every document must match. Accuracy (FR-013/SC-007)
depends on this; no doc should be finalized before it.

**⚠️ CRITICAL**: Complete before finalizing any user story content.

- [X] T002 Build the tool so commands/outputs can be verified: `cargo build --release` (or `nix build`); confirm the `shap` binary runs
- [X] T003 [P] Capture the authoritative command surface by running `shap --help` and `shap <cmd> --help` for `send agent model reasoning new status run read doctor`; record the verified commands/flags/examples as the accuracy baseline (working notes alongside `specs/003-project-documentation/quickstart.md`)
- [X] T004 [P] Extract the authoritative `:`↔`shap` mapping from `shell/zsh/shap.zsh` and the config/state facts from `docs/config.md` + `flake.nix` (outputs `packages.default`, `apps.default`, devshell) to reuse verbatim in new docs

**Checkpoint**: Verified fact baseline ready — user-story docs can be written against it.

---

## Phase 3: User Story 1 - Front page (Priority: P1) 🎯 MVP

**Goal**: A root `README.md` that conveys what `shap` is and the problem it solves within the first
screen, then routes readers onward (overview + links).

**Independent Test**: Show `README.md` to someone unfamiliar with `shap`; they can state what it is
and the core problem solved after reading only the opening section (SC-001).

- [X] T005 [US1] Create `README.md` with title, one-line description of `shap`, and a "What it does" core-capabilities list (shell-native ACP agent chat; agent/model/reasoning switching; `:run`/`:read` context; `:commit` helper; `doctor` self-check)
- [X] T006 [US1] Add to `README.md` a quick-install snippet (shortest path, linking to `docs/installation.md`) and exactly one minimal end-to-end usage example (configure one agent → `: hello` / `shap send "hello"`); do NOT include a full command reference (FR-002a)
- [X] T007 [US1] Add to `README.md` a Documentation links section pointing to `docs/index.md`, `docs/installation.md`, `docs/getting-started.md`, plus a license + one-line maturity caveat

**Checkpoint**: README delivers identity + navigation standalone (deep links may target files added in later phases).

---

## Phase 4: User Story 2 - Install from scratch (Priority: P1)

**Goal**: A reader on a clean machine reaches a working `shap` command via cargo or Nix.

**Independent Test**: On a clean environment, follow only `docs/installation.md`; `shap` runs and
reports version/help (SC-002).

- [X] T008 [US2] Create `docs/installation.md` with prerequisites (Rust toolchain per `rust-toolchain.toml` / `rust-version = 1.85`, or Nix with flakes) and Method A — build from source with cargo (`cargo build --release`, binary path, add to PATH)
- [X] T009 [US2] Add to `docs/installation.md` Method B — Nix flake (`nix run`, `nix profile install`, `nix develop`/direnv), the trade-offs between A and B, an install-verification step (`shap --version` / `shap doctor`), and a troubleshooting pointer (`shap doctor`, PATH note); document NO prebuilt-binary path (FR-003/R4)
- [X] T010 [P] [US2] Review/align `docs/nix.md` against `flake.nix`; fix any drift and ensure `docs/installation.md` links here for flake depth

**Checkpoint**: A clean-environment install succeeds end to end via both documented methods.

---

## Phase 5: User Story 3 - Core task via usage docs (Priority: P1)

**Goal**: A freshly-installed user completes the primary task (chat with an agent) by following the
getting-started walkthrough, with every command documented.

**Independent Test**: A user who only installed `shap` follows `docs/getting-started.md` and
completes the first task on the first attempt (SC-003), with all commands present (SC-004).

- [X] T011 [US3] Create `docs/getting-started.md`: prerequisite (link to installation), first-run config creating `config.toml` with a Claude Code agent block (`[agents.claude]`, `command = "claude-agent-acp"`, models `sonnet`/`opus`) noting any ACP agent works (link to `docs/agents.md`), and verification with `shap doctor`
- [X] T012 [US3] Add to `docs/getting-started.md` the first chat (`: hello` ≡ `shap send "hello"`, labeled equivalent) and a command tour covering every user-facing command (`send`, `agent`, `model`, `reasoning`/`effort`, `new`, `status`, `run`, `read`, `doctor`) — each with purpose + ≥1 verified example and its `shap <subcommand>` equivalent (FR-005/FR-008/SC-004)
- [X] T013 [P] [US3] Review/align `docs/agents.md` (agent config + ACP model) against the tool; add cross-links to/from getting-started
- [X] T014 [P] [US3] Review/align `docs/config.md` (fields, defaults, paths) against the current config schema; add cross-links
- [X] T015 [P] [US3] Review/align `docs/shell-integration.md`: confirm the `:`↔`shap` mapping table is current and authoritative (reused, not duplicated, by getting-started) and the "fully usable without the shell layer" note is present (FR-007)

**Checkpoint**: A new user can install and complete the primary task using only the docs.

---

## Phase 6: User Story 4 - Topic guides & index (Priority: P2)

**Goal**: A discoverable `docs/index.md` catalogs every guide; guides cross-link and are reachable
from the README.

**Independent Test**: From `docs/index.md`, a user with a specific question reaches the relevant
guide via a labeled link; no index entry is broken (SC-005).

- [X] T016 [US4] Create `docs/index.md` listing every guide with a one-line description and link: getting-started, installation, agents, config, shell-integration, nix (FR-009)
- [X] T017 [US4] Ensure each guide cross-links related guides (FR-010) and that every major topic is reachable from `README.md` within a few labeled links; no orphan documents (FR-014/SC-005)

**Checkpoint**: The whole doc set is navigable from the front page through the index.

---

## Phase 7: User Story 5 - Status & contribution (Priority: P3)

**Goal**: A reader can determine license, supported platforms, maturity, and where to file
issues/contribute.

**Independent Test**: From the docs alone, a reader states the license, platforms, maturity, and
issue channel (SC-006).

- [X] T018 [US5] State license (Apache-2.0, from `Cargo.toml`), supported platforms (macOS + Linux; zsh for the shell layer), and an honest pre-1.0 maturity/stability caveat in `README.md` (and reflected in `docs/index.md`)
- [X] T019 [US5] Add the issue/contribution channel (the GitHub repository `https://github.com/dgabka/shap`) to `README.md`

**Checkpoint**: Evaluators and contributors have license, platform, maturity, and contribution info.

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Enforce the accuracy and link contracts across the whole set (contract C6).

- [X] T020 [P] Accuracy gate (R1/R3/SC-007): verify every command, flag, path, and output across `README.md` and `docs/` matches the T003 baseline and `shell/zsh/shap.zsh`; fix any drift
- [X] T021 [P] Link-integrity gate (R2/SC-005): run the link-resolution helper from `quickstart.md`; fix every broken or orphaned link, including all `docs/index.md` entries
- [X] T022 Run the full `quickstart.md` verification checklist; perform the SC-001 unfamiliar-reader read and an SC-003 fresh follow-through of `docs/getting-started.md`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup. Produces the verified fact baseline that BLOCKS
  finalization of every story (drafting may start, but content must be verified against T003/T004).
- **User Stories (Phases 3–7)**: Depend on Foundational. US1–US3 are all P1; US4 (P2) benefits from
  US1–US3 existing (its index links point at them); US5 (P3) edits the README from US1.
- **Polish (Phase 8)**: Depends on all desired stories being drafted.

### User Story Dependencies

- **US1 (P1, README)**: Independent. MVP. Deep links may target later-phase files.
- **US2 (P1, install)**: Independent of other stories (relies on Foundational facts).
- **US3 (P1, usage)**: Independent; links to install but testable on its own.
- **US4 (P2, index)**: Soft dependency — its catalog entries resolve only once US1–US3 files exist;
  link-checking for US4 belongs to Polish (T021).
- **US5 (P3, metadata)**: Edits the README produced by US1.

### Within Each User Story

- Create the primary file before adding cross-links to it.
- Verify each command against the T003 baseline before publishing the example.

### Parallel Opportunities

- T003 and T004 (Foundational) run in parallel — different sources.
- T010, T013, T014, T015 (existing-guide reviews) are all `[P]` — distinct files, no shared edits.
- T020 and T021 (Polish gates) run in parallel — read-only checks over the set.
- With multiple authors, US1/US2/US3 can be drafted concurrently after Foundational (distinct files:
  `README.md`, `docs/installation.md`, `docs/getting-started.md`).

---

## Parallel Example: Foundational + existing-guide reviews

```bash
# Foundational facts (Phase 2):
Task: "Capture command surface from shap --help / subcommand --help"   # T003
Task: "Extract :-to-shap mapping and config/flake facts"               # T004

# Existing-guide reviews (can run together once facts are captured):
Task: "Review/align docs/nix.md vs flake.nix"            # T010
Task: "Review/align docs/agents.md"                      # T013
Task: "Review/align docs/config.md"                      # T014
Task: "Review/align docs/shell-integration.md"           # T015
```

---

## Implementation Strategy

### MVP First (User Story 1 only)

1. Phase 1 Setup → Phase 2 Foundational (capture verified facts).
2. Phase 3 US1 → `README.md` front page.
3. **STOP and VALIDATE**: unfamiliar-reader test (SC-001). README alone is a shippable improvement
   (the repo currently has no README).

### Incremental Delivery

1. Foundation ready (Phases 1–2).
2. US1 README → validate SC-001 → ship (MVP).
3. US2 installation → validate SC-002 → ship.
4. US3 getting-started + guide reviews → validate SC-003/SC-004 → ship.
5. US4 index/navigation → validate SC-005.
6. US5 metadata → validate SC-006.
7. Polish gates (T020–T022) → validate SC-007 across the set.

### Parallel Team Strategy

After Foundational: Author A → US1 (`README.md`); Author B → US2 (`docs/installation.md`); Author
C → US3 (`docs/getting-started.md` + guide reviews). Index (US4) and Polish follow once files land.

---

## Notes

- [P] = different files, no dependency on an incomplete task.
- No source code changes; `crates/`, `shell/`, `flake.nix` are read-only sources of truth.
- "Verify before publish": every command/flag/output must match the running tool (T002/T003).
- Commit after each task or logical group.
- Stop at any checkpoint to validate a story independently.
