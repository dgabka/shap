# Shell integration (Zsh)

The shell layer is intentionally thin: it maps `:` commands to the `shap` binary, forwards the current
directory, renders an optional prompt segment, and inserts the generated `git commit` line into the
buffer. All product logic lives in `shap` — every `:` command has a direct `shap <subcommand>`
equivalent, so the tool is fully usable without the shell layer.

## Install

```sh
echo 'source /path/to/shap/shell/zsh/shap.zsh' >> ~/.zshrc
exec zsh
```

Override the binary location before sourcing if needed:

```sh
export SHAP_BIN=/path/to/shap
```

If `shap` is not on PATH, the integration prints a notice and stays inactive.

## Commands

| Typed | Runs |
|-------|------|
| `: <prompt>` | `shap send "<prompt>"` (colon + space) |
| `:agent [name]` | `shap agent [name]` |
| `:model [name]` | `shap model [name]` |
| `:reasoning [level]` / `:effort [level]` | `shap reasoning [level]` |
| `:new` | `shap new` |
| `:status` | `shap status` |
| `:run <cmd…>` | `shap run -- <cmd…>` |
| `:read <prompt>` | `shap read "<prompt>"` |
| `:commit` | prefills the buffer with `git commit -am "…"` (never runs it) |
| `:doctor` | `shap doctor` |

The bare `: <prompt>` chat is an `accept-line` widget that triggers **only** on a leading colon-space,
so ordinary uses of the `:` builtin (e.g. `: ${VAR:=default}`) are untouched. The `:cmd` forms are
plain functions whose names contain `:` and never collide with the builtin.

Every `:cmd` — including `:commit` — is defined as a function, so command-word highlighters
(zsh-syntax-highlighting, fast-syntax-highlighting) resolve it and render it as a valid command rather
than flagging it red as unknown. `:commit`'s behavior is still driven by the `accept-line` widget (see
below); its function exists only so the word resolves for highlighting, and it prints usage guidance
(never runs git) if invoked with arguments like `:commit foo`.

## Prompt segment

A `precmd` hook caches the segment once per prompt into `$SHAP_PROMPT_INFO` (it reads only
`state.json`, so it is cheap). Add it to your prompt:

```sh
# left prompt
PROMPT='%~ ${SHAP_PROMPT_INFO} %# '
# or right prompt
RPROMPT='${SHAP_PROMPT_INFO}'
```

The segment renders as `[shap codex·gpt-5·high]`, or nothing when no selection is set. Disable it with:

```sh
export SHAP_PROMPT_SEGMENT=0
```

(also honour `[ui].show_prompt_segment` in your config for the tool side.)

## `:commit` widget

Typing `:commit` and pressing Enter runs `shap commit --prefill-shell-buffer`, captures the single
`git commit -am "…"` line, and replaces the buffer with it for review. **It never executes the commit**
— you edit and press Enter yourself (Constitution VII). Errors (not a repo, nothing to commit) are
shown via a ZLE message. The widget intercepts a bare `:commit` before the `:commit` function would
run, so the function (which only resolves the word for highlighting) never fires in this path.

## Completions

```sh
shap completions zsh  > "${fpath[1]}/_shap"   # or bash/fish/…
```

## See also

- [Getting started](./getting-started.md) · [Configuration](./config.md) ·
  [Documentation index](./index.md)
