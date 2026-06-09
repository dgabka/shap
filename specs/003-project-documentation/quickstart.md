# Quickstart: Authoring & Verifying the Documentation

This is the working procedure for producing and checking the documentation set defined by this
plan. It is for the author of the docs, not an end user.

## Prerequisites

- A working `shap` build (so commands/outputs can be verified): `cargo build --release` or
  `nix build`.
- The four existing guides under `docs/` and the contract in
  `contracts/docs-structure.md` open for reference.

## Authoring order

Follow the dependency order so links point at files that already exist:

1. **`docs/installation.md`** — write the two install methods; verify each command against
   `flake.nix` and the cargo build. Confirm `shap --version` / `shap doctor` work as shown.
2. **`docs/getting-started.md`** — write the Claude Code first-run walkthrough and the command
   tour. For each command, run `shap <cmd> --help` and confirm the documented purpose/args/example.
3. **Review existing guides** — `agents.md`, `config.md`, `shell-integration.md`, `nix.md`: fix any
   drift, ensure cross-links, confirm the shell-integration mapping table and the "usable without
   the shell layer" note are present and current.
4. **`docs/index.md`** — list every guide with a one-line description; confirm each link resolves.
5. **`README.md`** — write last, so it can link to finished pages. Keep it overview-scoped.

## Verification (run before considering the feature done)

Source-of-truth command list to verify coverage against:

```sh
# Enumerate the real command surface
shap --help
for c in send agent model reasoning new status run read doctor; do shap "$c" --help; done
```

Checks (map to contract C1–C6 and spec Success Criteria):

- [ ] All eight required files from contract C1 exist.
- [ ] Every command/flag/output in the docs appears in the `--help` output above or in
  `shell/zsh/shap.zsh` (no drift). [R1, SC-007]
- [ ] Every user-facing command is documented with purpose + ≥1 example. [R3, SC-004]
- [ ] Every relative link and every `docs/index.md` entry resolves to an existing file/anchor.
  [R2, SC-005]
- [ ] Installation documents cargo + Nix only; no prebuilt-binary instructions. [R4]
- [ ] README has no full command reference; depth reached via links. [R5]
- [ ] Dual-surface commands show both `:` and `shap` forms, labeled equivalent. [R6]
- [ ] License, supported platforms, maturity caveat, and issue channel are present. [R7, SC-006]

Quick link-resolution helper (relative Markdown links):

```sh
# From repo root: list link targets that don't exist on disk (anchors excluded)
grep -roE '\]\(([^)]+\.md[^)]*)\)' README.md docs/ \
  | sed -E 's/.*\(([^)#]+).*/\1/' | sort -u \
  | while read -r f; do [ -e "$f" ] || [ -e "docs/$f" ] || echo "MISSING: $f"; done
```

## Done signal

All checklist boxes above are ticked, the unfamiliar-reader test for SC-001 passes, and a clean
follow-through of `getting-started.md` completes the first task on the first attempt (SC-003).
