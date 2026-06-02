# Feature Specification: Nix Flake — Dev Shell, Package, and App

**Feature Branch**: `002-flake-nix-devshell`

**Created**: 2026-06-03

**Status**: Draft

**Input**: User description: "flake.nix, provide devshell for developers and the package/app."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reproducible developer environment (Priority: P1)

A contributor clones the repository and enters a single command to obtain a fully provisioned
development environment: the pinned Rust toolchain, the build and test tooling, and the optional
runtime helpers the project expects. They can immediately build, test, format, and lint without
manually installing or version-matching anything on their host.

**Why this priority**: The dev shell removes the largest source of contributor friction — "works on
my machine" toolchain drift. It is the foundation the package and app builds reuse, and it delivers
value even if no packaged artifact is ever produced. This is the MVP.

**Independent Test**: From a clean checkout on a machine that has only the flake toolchain installed,
enter the dev shell and run the project's build and test commands successfully, without installing any
project-specific tooling by hand.

**Acceptance Scenarios**:

1. **Given** a clean checkout and the flake toolchain installed, **When** the contributor enters the
   dev shell, **Then** the pinned Rust toolchain and all declared build/test tools are available on
   PATH at the versions the project requires.
2. **Given** an active dev shell, **When** the contributor runs the workspace build and the test
   suite, **Then** both complete using only tools provided by the shell.
3. **Given** an active dev shell, **When** the contributor runs the project's formatter and linter,
   **Then** both run successfully without additional installation.

---

### User Story 2 - Buildable package artifact (Priority: P2)

A user or packager builds the project from source through the flake and receives the `shap` binary as
a self-contained, reproducible result, without needing a pre-provisioned developer environment.

**Why this priority**: A reproducible package build is what lets others install, distribute, and
cache the tool. It depends on the same toolchain pinning as the dev shell but targets consumers rather
than contributors.

**Independent Test**: On a clean machine with only the flake toolchain, build the package from the
repository and confirm a runnable `shap` binary is produced in the build result.

**Acceptance Scenarios**:

1. **Given** a clean machine with the flake toolchain, **When** the user builds the package, **Then**
   the build succeeds and produces a runnable `shap` executable in the result.
2. **Given** two builds of the same revision on supported systems, **When** they are compared, **Then**
   they produce equivalent results (inputs are pinned; no reliance on host-installed toolchains).

---

### User Story 3 - Run the app without installing (Priority: P3)

A user runs `shap` directly through the flake — for a quick trial or one-off invocation — without
adding it to their profile or building it into a checkout first.

**Why this priority**: Frictionless trial lowers the barrier to adoption, but it is a convenience over
the package build (US2), which already yields a runnable binary.

**Independent Test**: On a clean machine with the flake toolchain, invoke the app through the flake's
run entry point and confirm `shap` starts and responds (e.g. prints version/help).

**Acceptance Scenarios**:

1. **Given** the flake toolchain installed, **When** the user runs the app entry point with no
   sub-command, **Then** `shap` starts and shows its usage/help without error.
2. **Given** the flake toolchain installed, **When** the user runs the app's self-check (`doctor`),
   **Then** it executes and reports environment status.

---

### Edge Cases

- What happens when the build is attempted on an unsupported system (e.g. Windows, or an
  architecture not in the supported set)? The flake MUST surface a clear, early message rather than
  failing obscurely deep in the build.
- How does the build behave when the workspace dependency lock is stale or inconsistent with the
  manifest? The build MUST fail with a message that points at the lock mismatch rather than silently
  fetching unpinned dependencies.
- What happens when a contributor's host already has a conflicting Rust toolchain installed? The dev
  shell MUST take precedence so the project's pinned toolchain is the one used.
- How are optional runtime helpers (picker, git, ACP agents) handled when absent? They are optional;
  the dev shell provides the ones it can, and their absence MUST NOT block build or test.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The repository MUST provide a single flake entry point at the repository root that
  exposes the dev shell, the package, and the runnable app.
- **FR-002**: The dev shell MUST provide the project's pinned Rust toolchain at the version the
  workspace requires (currently Rust 1.85 / edition 2024), so contributors do not match it by hand.
- **FR-003**: The dev shell MUST provide the build and test tooling the project uses (workspace build,
  the test runner, formatter, and linter) such that the documented build/test/format/lint commands
  succeed using only shell-provided tools.
- **FR-004**: The dev shell SHOULD provide the optional runtime helpers used by the tool (e.g. a fuzzy
  picker and `git`) so contributors can exercise the full feature set, while their absence MUST NOT
  block building or testing.
- **FR-005**: The flake MUST expose a package build that produces a runnable `shap` binary from the
  Cargo workspace.
- **FR-006**: The package build MUST be reproducible — it MUST pin all inputs (toolchain and
  dependencies) and MUST NOT depend on tooling installed on the host outside the flake.
- **FR-007**: The flake MUST expose a runnable app entry point that launches the built `shap` binary
  directly.
- **FR-008**: The flake MUST declare the systems it supports and MUST cover the project's MVP targets
  (macOS and Linux on both `aarch64` and `x86_64`); invoking it on an unsupported system MUST fail
  with a clear message.
- **FR-009**: The flake's pinned inputs MUST be recorded in a committed lock file so every consumer
  resolves identical inputs.
- **FR-010**: The default package and default app of the flake MUST be `shap`, so the conventional
  no-argument build and run commands operate on the project binary.
- **FR-011**: The package build MUST consume the workspace's existing dependency lock and MUST fail
  loudly if that lock is missing or inconsistent, rather than fetching unpinned dependencies.
- **FR-012**: Contributor-facing documentation MUST describe how to enter the dev shell, build the
  package, and run the app via the flake.
- **FR-013**: The dev shell and package build MUST share the same pinned toolchain definition so the
  environment a contributor tests in matches the environment the package is built in.

### Key Entities *(include if feature involves data)*

- **Flake**: The single declarative entry point at the repository root. Exposes a dev shell, a
  package, and an app; declares supported systems; references pinned inputs.
- **Pinned inputs / lock**: The committed record of exact input revisions (toolchain source,
  dependency sources) that makes every resolution identical across machines.
- **Dev shell**: The provisioned environment (toolchain + build/test/lint tooling + optional
  helpers) a contributor enters to work on the project.
- **Package**: The reproducible build output — the `shap` binary produced from the Cargo workspace.
- **App**: The runnable entry point that launches the packaged `shap` binary.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A contributor on a clean machine goes from checkout to a working development environment
  with a single command, with no manual toolchain or tool installation.
- **SC-002**: 100% of the project's documented build, test, format, and lint commands succeed using
  only tooling the dev shell provides.
- **SC-003**: Building the package on a clean machine (no host Rust toolchain) succeeds and yields a
  runnable `shap` binary.
- **SC-004**: Two builds of the same revision on the same supported system produce equivalent results,
  demonstrating reproducibility.
- **SC-005**: A user can run `shap` directly through the flake and reach usage/help on the first
  attempt without any prior build or install step.
- **SC-006**: All four MVP target systems (macOS and Linux × `aarch64` and `x86_64`) are declared and
  build the package successfully.

## Assumptions

- Target consumers are developers/packagers comfortable using the Nix package manager with flakes
  enabled; teaching Nix itself is out of scope.
- The supported systems mirror the MVP targets in the project plan (macOS and Linux on `aarch64` and
  `x86_64`); Windows is out of scope, consistent with the existing plan.
- The pinned Rust toolchain version tracks the workspace manifest (`rust-version`/edition); when the
  manifest changes, the flake's toolchain pin is expected to follow.
- The package build reuses the committed `Cargo.lock`; dependency versions are governed there, not
  re-decided by this feature.
- Optional runtime helpers (fuzzy picker, ACP agent adapters, git) are conveniences for the dev shell;
  the feature does not require bundling third-party ACP agents.
- A continuous-integration system that consumes the flake (e.g. to build or check it) is desirable but
  out of scope for this specification, which covers the flake's developer-facing outputs only.
