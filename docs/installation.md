# Installation

`shap` runs on macOS and Linux (`aarch64`/`x86_64`). Windows is out of scope. The optional shell
integration targets Zsh, but the `shap` binary is fully usable on its own.

There are two supported ways to install: **build from source with cargo**, or **use the Nix flake**.

| Method | Best when | Trade-off |
|--------|-----------|-----------|
| Cargo (from source) | You already have a Rust toolchain and want a plain binary on `PATH`. | You manage the toolchain and updates yourself. |
| Nix flake | You have Nix and want a reproducible build with the pinned toolchain and ACP adapters provided. | Requires Nix with flakes enabled. |

---

## Method A — Build from source with cargo

### Prerequisites

- A Rust toolchain matching the repository pin in [`rust-toolchain.toml`](../rust-toolchain.toml)
  (currently Rust **1.88.0**). If you use `rustup`, the pin is picked up automatically inside the
  repo.
- A C toolchain/linker (standard build essentials for your platform).

### Build and install

```sh
git clone https://github.com/dgabka/shap
cd shap

# Option 1: install into ~/.cargo/bin (must be on your PATH)
cargo install --path crates/shap-cli

# Option 2: build a release binary in-tree
cargo build --release
# → target/release/shap   (copy it somewhere on your PATH)
```

---

## Method B — Nix flake

### Prerequisites

- Nix with flakes enabled (`experimental-features = nix-command flakes`).

Nothing else — the flake provides the pinned Rust toolchain and the ACP adapters.

### Install or run

```sh
# Run without installing
nix run github:dgabka/shap -- --help

# Build the package locally
nix build            # → ./result/bin/shap
./result/bin/shap --version

# Install into your Nix profile
nix profile install github:dgabka/shap
```

For the developer environment, package details, and supported systems, see [nix.md](./nix.md).

---

## Verify the install

```sh
shap --version
shap doctor
```

`shap doctor` validates the installation and your configured agents and prints a remediation line
for each problem, for example:

```text
[FAIL] config: no config file at ~/.config/shap/config.toml
[ok  ] git: available
[ok  ] sessions: directory is writable
[warn] shell: integration not detected; source shell/zsh/shap.zsh
```

A missing config is expected on a fresh install — continue to
[Getting started](./getting-started.md) to create one.

## Troubleshooting

- **`shap: command not found`** — the binary isn't on your `PATH`. For cargo, ensure `~/.cargo/bin`
  (Option 1) or your chosen directory (Option 2) is on `PATH`. For Nix, use `nix profile install` or
  call `./result/bin/shap` directly.
- **Anything else** — run `shap doctor` and follow the remediation line for each failing check.

## Next steps

- [Getting started](./getting-started.md) — configure an agent and send your first prompt.
- [Shell integration](./shell-integration.md) — optional Zsh `:` commands.
- [Documentation index](./index.md) — all guides.
