# Implementation Plan: Project Documentation

**Branch**: `003-project-documentation` | **Date**: 2026-06-09 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-project-documentation/spec.md`

## Summary

Deliver the project's user-facing documentation set: a root `README.md` front page (overview +
links, per clarification) plus a completed, indexed `docs/` directory. The work organizes and fills
gaps in the four existing topic guides (`agents.md`, `config.md`, `shell-integration.md`, `nix.md`),
adds the two missing pieces — the front page and a getting-started walkthrough — and a `docs/`
index. Install docs cover cargo-from-source and the Nix flake only; the getting-started walkthrough
uses Claude Code as the worked example agent. Deliverable is in-repo Markdown; no site generator,
no API-ref tooling.

## Technical Context

**Language/Version**: Markdown (CommonMark + GitHub-flavored tables/task-lists). No code changes to
the Rust workspace.

**Primary Dependencies**: None added. Source-of-truth for examples is the existing `shap` CLI
(subcommands `send`, `agent`, `model`, `reasoning`, `new`, `status`, `run`, `read`, `doctor`),
the zsh integration in `shell/zsh/shap.zsh`, the Nix flake outputs, and the existing `docs/` guides.

**Storage**: Files in the repository — `README.md` at root and Markdown under `docs/`.

**Testing**: Manual verification against the spec's acceptance scenarios plus a mechanical
link/command-accuracy check: every documented command/flag must exist in the CLI, and every
intra-doc link must resolve. No automated test harness is added (constitution: YAGNI).

**Target Platform**: Repository host's Markdown renderer (GitHub) for the README; plain Markdown
for `docs/`. Documented install/run targets are macOS and Linux (the shell integration is zsh).

**Project Type**: Documentation for a Rust CLI + thin zsh shell integration.

**Performance Goals**: N/A (static documentation).

**Constraints**: No content drift — every command, flag, path, and output shown MUST match the tool
as it exists at publication (FR-013/SC-007). README stays overview-scoped and links into `docs/`
rather than duplicating reference content (FR-002a). English-only; in-repo Markdown only.

**Scale/Scope**: 1 new front page (`README.md`), 1 new getting-started guide, 1 `docs/` index, and
review/gap-fill passes over 4 existing guides. ~6 documentation files touched or created.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The constitution is code-oriented; the applicable principles map to documentation as follows:

| Principle | Application to this feature | Status |
|-----------|------------------------------|--------|
| I. Keep It Simple | Plain Markdown, no site generator or doc framework. README links instead of duplicating. | PASS |
| II. Keep It Lean (YAGNI) | No hosted site, no API-ref generation, no i18n, no prebuilt-binary install path. Only docs with a present need. | PASS |
| III. Code Quality (→ doc quality) | Consistent voice, headings, and command formatting across guides; reuse existing guide structure. | PASS |
| VI. Fail Clearly | Install/usage docs reference `shap doctor` and troubleshooting for failure diagnosis (FR-011). | PASS |
| VII. Keep User Control | Docs state the tool is fully usable without the shell integration (FR-007). | PASS |
| VIII. Respect the Shell | Shell-integration doc already documents non-invasive `:`-command behavior; front page must not overstate. | PASS |
| IX. Minimize Dependencies | Zero new tooling/dependencies introduced. | PASS |
| X. Preserve Contributor Clarity | Honest pre-1.0 maturity caveat and contribution/issue channel stated (FR-012). | PASS |

No violations. Complexity Tracking not required.

## Project Structure

### Documentation (this feature)

```text
specs/003-project-documentation/
├── plan.md              # This file (/speckit-plan command output)
├── research.md          # Phase 0 output — doc structure & format decisions
├── data-model.md        # Phase 1 output — document inventory & relationships
├── quickstart.md        # Phase 1 output — author + verify workflow
├── contracts/
│   └── docs-structure.md # Phase 1 output — required files, sections, link graph
├── checklists/
│   └── requirements.md   # Spec quality checklist (from /speckit-specify)
└── tasks.md             # Phase 2 output (/speckit-tasks — NOT created here)
```

### Source Code (repository root)

This feature changes documentation only. The relevant repository layout (existing unless marked
NEW):

```text
README.md                      # NEW — front page (overview + links)
docs/
├── index.md                   # NEW — documentation index / catalog
├── getting-started.md         # NEW — fresh-install → first task walkthrough (Claude Code example)
├── installation.md            # NEW — cargo-from-source + Nix flake (consolidates install steps)
├── agents.md                  # EXISTING — review/align (agent config, ACP model)
├── config.md                  # EXISTING — review/align (config + state file reference)
├── shell-integration.md       # EXISTING — review/align (zsh `:` commands)
└── nix.md                     # EXISTING — review/align (flake devshell/package/app)

crates/        # source of truth for documented commands — NOT modified
shell/zsh/     # source of truth for documented `:` commands — NOT modified
flake.nix      # source of truth for documented Nix install — NOT modified
```

**Structure Decision**: Keep the project's established convention — a root `README.md` as the
overview front page plus a flat `docs/` directory of topic guides. Add the two missing entry
documents (`getting-started.md`, `installation.md`) and a `docs/index.md` catalog, then review the
four existing guides for accuracy and cross-linking. A flat `docs/` directory (no nested
subfolders, no site generator) satisfies KISS/YAGNI at the current document count.

## Complexity Tracking

> No constitution violations. Section intentionally empty.
