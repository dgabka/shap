# Contract: Documentation Structure

The "interface" this feature exposes is the documentation set a reader navigates. This contract
defines the required files, the required sections within each, and the link graph — the checkable
shape the deliverable must satisfy. Phase 2 tasks and the quality checklist verify against it.

## C1 — Required files exist

| Path | State | Must contain |
|------|-------|--------------|
| `README.md` | NEW | identity, capabilities, quick install, one usage example, links, license+maturity |
| `docs/index.md` | NEW | one indexed entry per topic guide, each with a one-line description |
| `docs/installation.md` | NEW | prerequisites, cargo method, Nix method, trade-offs, verify, troubleshooting |
| `docs/getting-started.md` | NEW | first-run config (Claude Code), verify, first chat, command tour |
| `docs/agents.md` | EXISTING | reviewed/current; linked from index |
| `docs/config.md` | EXISTING | reviewed/current; linked from index |
| `docs/shell-integration.md` | EXISTING | reviewed/current; `:`↔`shap` table; "usable without shell" note |
| `docs/nix.md` | EXISTING | reviewed/current; linked from installation for depth |

## C2 — README contract

- MUST state what `shap` is within the first screen (before any deep section). [FR-001, SC-001]
- MUST list core capabilities. [FR-002]
- MUST provide a quick-install snippet and link to `docs/installation.md`. [FR-002]
- MUST include exactly one minimal end-to-end usage example. [FR-002a]
- MUST NOT contain a full command reference (that lives in `docs/`). [FR-002a]
- MUST link to `docs/index.md`, `docs/installation.md`, `docs/getting-started.md`. [FR-014]
- MUST state license + a one-line maturity caveat. [FR-012]

## C3 — Installation contract

- MUST document prerequisites for each method. [FR-003]
- MUST document exactly two methods: cargo-from-source and Nix flake. [FR-003]
- MUST NOT document prebuilt-binary download/install. [FR-003, R4]
- MUST state the trade-offs between the two methods. [FR-003]
- MUST include an install-verification step (`shap --version` and/or `shap doctor`). [SC-002]
- MUST reference troubleshooting (`shap doctor`). [FR-011]
- Commands MUST match `flake.nix` outputs and the pinned toolchain. [R1]

## C4 — Getting-started contract

- MUST take the reader from a fresh install to a completed first task. [FR-004, SC-003]
- MUST use Claude Code as the worked example agent and note any ACP agent works. [FR-004]
- MUST document every user-facing command with purpose + ≥1 example:
  `send`, `agent`, `model`, `reasoning`/`effort`, `new`, `status`, `run`, `read`, `doctor`.
  [FR-005, SC-004]
- MUST present dual-surface commands in both `:` and `shap` forms, labeled equivalent. [FR-008]
- MUST reference required/optional configuration with defaults (or link to `config.md`). [FR-006]

## C5 — Index & navigation contract

- `docs/index.md` MUST list every topic guide with a one-line description. [FR-009]
- Every index entry MUST resolve to an existing file. [SC-005]
- Every major topic MUST be reachable from `README.md` within a small number of labeled links.
  [FR-014]
- No document may be orphaned (unreachable from the README link graph). [SC-005]

## C6 — Accuracy contract (global)

- Every command, flag, path, and output shown MUST match the tool at publication time. [FR-013, SC-007]
- Every relative link MUST resolve. [SC-005]
- Each topic guide MUST be self-contained for its topic and cross-link related guides. [FR-010]

## Acceptance check (maps to spec Success Criteria)

| Check | Method | Criterion |
|-------|--------|-----------|
| Identity legible from front page | Read README opening; unfamiliar-reader test | SC-001 |
| Clean-env install works | Follow installation.md on clean env; run `shap` | SC-002 |
| First task succeeds first try | Follow getting-started.md end to end | SC-003 |
| 100% command coverage | Diff documented commands vs `shap --help` set | SC-004 |
| No broken/orphan links | Resolve every relative link + index entry | SC-005 |
| Metadata present | Grep for license/platforms/maturity/issue channel | SC-006 |
| No drift | Cross-check every command/flag/output vs tool | SC-007 |
