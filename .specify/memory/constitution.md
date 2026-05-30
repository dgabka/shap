<!--
SYNC IMPACT REPORT
==================
Version change: (unversioned template) → 1.0.0
Rationale: Initial ratification. All template placeholders replaced with concrete
principles and governance. MAJOR baseline established.

Modified principles:
  [PRINCIPLE_1..5 placeholders] → expanded to 10 named principles:
    I. Keep It Simple (KISS)
    II. Keep It Lean (YAGNI)
    III. Code Quality
    IV. Tests for Meaningful Logic
    V. Readability Over Performance Tricks
    VI. Fail Clearly
    VII. Keep User Control
    VIII. Respect the Shell
    IX. Minimize Dependencies
    X. Preserve Contributor Clarity

Added sections:
  - Core Principles (10 principles; template shipped with 5 slots)
  - Additional Constraints (Architecture & Technology)
  - Development Workflow & Quality Gates
  - Governance

Removed sections: none

Templates requiring updates:
  ✅ .specify/templates/plan-template.md     — generic Constitution Check gate, aligned
  ✅ .specify/templates/spec-template.md      — no principle references, aligned
  ✅ .specify/templates/tasks-template.md     — no principle references, aligned
  ✅ .specify/templates/checklist-template.md — no principle references, aligned

Follow-up TODOs:
  - RATIFICATION_DATE set to 2026-05-30 (initial adoption; no prior date existed).
    Confirm with maintainers if a different formal adoption date applies.
-->

# Shap Constitution

## Core Principles

### I. Keep It Simple (KISS)

Prefer straightforward, readable solutions over clever abstractions. Implementations MUST
avoid premature generalization, unnecessary layers, framework-like architecture, and
speculative extensibility. When two approaches solve the same current requirement, the
simpler one MUST be chosen unless a concrete, documented reason justifies the more complex
one.

Rationale: This is an open-source project. Simple code is the cheapest code for a new
contributor to read, trust, and change. Cleverness is a tax paid on every future read.

### II. Keep It Lean (YAGNI)

Code, dependencies, modules, traits, configuration options, and abstractions MUST be added
only when they serve a clear current requirement. Infrastructure for hypothetical future
use cases MUST NOT be added. Configuration knobs and extension points MUST be justified by
a real, present need.

Rationale: Unused flexibility is dead weight that contributors must still understand and
maintain. Lean surface area keeps the project navigable and reduces breakage.

### III. Code Quality

Implementation MUST be idiomatic for its language, well-structured, and consistent with the
surrounding code. Names MUST be clear and intention-revealing. Functions MUST stay focused
on a single responsibility. Error handling MUST be explicit. Hidden behavior and surprising
side effects MUST NOT be introduced.

Rationale: Consistent, explicit code lowers the cognitive cost of every contribution and
makes review meaningful rather than archaeological.

### IV. Tests for Meaningful Logic

Unit tests are REQUIRED for implementation logic, branching behavior, edge cases, and error
handling. Simple data definitions, trivial pass-through functions, and purely declarative
configuration do NOT require tests unless they affect behavior. Tests MUST be deterministic
and MUST assert behavior, not implementation details.

Rationale: Tests on meaningful logic protect contributors from silent regressions; tests on
trivial declarations add maintenance cost without protective value.

### V. Readability Over Performance Tricks

Readable code MUST be the default. Complex optimization, caching, concurrency, and low-level
tricks MUST NOT be introduced unless there is a demonstrated problem or unless poor
performance would noticeably harm the shell user experience. Any such optimization MUST be
justified in the change description.

Rationale: Most performance "improvements" are speculative and degrade clarity. Optimize
only against measured problems that users actually feel.

### VI. Fail Clearly

Errors MUST be actionable and human-readable. When something is misconfigured, unavailable,
unsupported, or unsafe, the tool MUST explain what happened and what the user can do next.
Error messages MUST NOT expose raw internal failures without context.

Rationale: A CLI's error messages are its primary support channel. Clear failures turn a
dead end into a next step.

### VII. Keep User Control

The tool MUST NOT execute destructive actions automatically. Generated commands, especially
Git commands, MUST be shown to the user and MUST require explicit confirmation or manual
execution before any state-changing operation runs.

Rationale: Trust is the product. Users must always see and approve what touches their
repositories and their machines.

### VIII. Respect the Shell

Shell integration MUST remain lightweight. It MUST NOT measurably slow down shell startup,
prompt rendering, or command execution. Complex logic MUST live in the Rust CLI, not in
shell scripts; shell scripts MUST stay thin wrappers that delegate to the CLI.

Rationale: A shell tool that adds latency to every prompt becomes a tool users disable.
Keeping logic in the compiled CLI keeps the interactive path fast.

### IX. Minimize Dependencies

Dependencies MUST be added only when they solve a real problem better than local code would.
Mature, focused crates are PREFERRED. Large dependencies MUST NOT be pulled in for small
conveniences. Each new dependency MUST be justified in the change that introduces it.

Rationale: Every dependency is supply-chain surface, build cost, and a thing contributors
must learn. Fewer, well-chosen dependencies keep the project auditable.

### X. Preserve Contributor Clarity

Every module MUST have an obvious, single purpose. Patterns that make the project harder to
navigate — excessive indirection, global mutable state, hidden macros, and overly generic
traits — MUST be avoided. New contributors MUST be able to locate where behavior lives by
reading module names and structure.

Rationale: An open-source project lives or dies by how quickly a newcomer can orient. Clear
structure is a feature, not a nicety.

## Additional Constraints

**Architecture & Technology**

- The core logic MUST reside in the Rust CLI. Shell scripts are limited to thin integration
  glue (sourcing, hooks, delegation to the CLI binary).
- The interactive shell path (startup, prompt, per-command hooks) MUST stay free of heavy or
  blocking work; expensive operations belong behind explicit user-invoked commands.
- Destructive or state-changing operations (notably Git operations) MUST be surfaced for user
  review before execution per Principle VII.

These constraints operationalize Principles VII, VIII, and IX and are non-negotiable for any
shell-facing or repository-touching code path.

## Development Workflow & Quality Gates

- **Review**: Every change MUST be reviewable against the Core Principles. Reviewers MUST
  flag violations of KISS (I), YAGNI (II), and Contributor Clarity (X) as blocking unless
  justified.
- **Testing gate**: Changes that add or modify meaningful logic MUST include unit tests per
  Principle IV. CI MUST run the test suite, and failing tests MUST block merge.
- **Dependency gate**: Any pull request adding a dependency MUST state why local code is
  insufficient (Principle IX).
- **Performance gate**: Any optimization that reduces readability MUST cite the measured
  problem it addresses (Principle V).
- **Complexity justification**: Deviations from these principles MUST be documented in the
  change description with the reason and the simpler alternative that was rejected.

## Governance

This constitution supersedes other development practices for the project. When guidance
conflicts, the constitution wins.

**Amendment procedure**: Amendments MUST be proposed via pull request that updates this
document, states the rationale, and updates the version and dates below. Amendments take
effect once merged.

**Versioning policy**: This document follows semantic versioning:
- **MAJOR**: Backward-incompatible governance changes — removing or redefining a principle.
- **MINOR**: Adding a new principle or section, or materially expanding guidance.
- **PATCH**: Clarifications, wording fixes, and non-semantic refinements.

**Compliance review**: All pull requests and code reviews MUST verify compliance with the
Core Principles. Unjustified violations MUST NOT be merged. Justified deviations MUST be
recorded in the change description per the Complexity justification gate.

**Version**: 1.0.0 | **Ratified**: 2026-05-30 | **Last Amended**: 2026-05-30
