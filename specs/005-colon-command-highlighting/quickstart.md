# Quickstart: Colon-Command Syntax Highlighting

Manual build + verification. No Rust change, so the focus is the zsh integration behavior.

## Build / activate

```sh
cargo build                      # CLI unchanged; build just to have `shap` on PATH
export SHAP_BIN="$(pwd)/target/debug/shap"
source shell/zsh/shap.zsh
```

Enable a command-word highlighter in the test shell (either one):

```sh
# zsh-syntax-highlighting
source /path/to/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh
# or fast-syntax-highlighting
source /path/to/fast-syntax-highlighting/fast-syntax-highlighting.plugin.zsh
```

(Re-source `shap.zsh` after the highlighter if your highlighter snapshots functions at load — order
should not matter for function resolution, but re-sourcing is a safe check.)

## Verify highlighting (SC-001, SC-002, US1/US2)

1. Type `:commit` (do **not** press Enter). It must render in the **recognized-command** style — the
   same color as `:agent` / `:status` — not red/underlined as an unknown command.
2. Type each of `:agent`, `:model`, `:reasoning`, `:effort`, `:new`, `:status`, `:doctor`, `:run`,
   `:read`. None should render as unknown.

Expected: no colon command shows the invalid/unknown style. Before the fix, `:commit` was red.

## Verify behavior is unchanged (SC-003, FR-002)

3. In a git repo with staged/unstaged changes, type `:commit` and press **Enter**. The line must be
   replaced by a single `git commit -am "…"` line for review. Nothing is committed automatically.
   Press Enter again yourself only if you want to commit.
4. In a non-repo (or with nothing to commit), type `:commit` + Enter. A clear ZLE message explains the
   problem; no commit happens.

## Verify misuse guidance (FR-006/VI, B2)

5. Type `:commit something` and press Enter. Expected: a short, actionable message telling you to type
   `:commit` (no args) and press Enter. No git command runs. (Before: "command not found".)

## Verify no regression on the `:` paths (SC-004, FR-004)

6. Type `: hello world` + Enter → runs `shap send "hello world"` as before.
7. Type `: ${FOO:=bar}` (bare-colon builtin use) → behaves as the zsh `:` builtin, untouched.

## Done when

- `:commit` and all colon commands render as recognized (not red).
- `:commit` + Enter still only prefills the commit line; never auto-commits.
- `: <text>` chat and bare `:` builtin behavior are unchanged.
