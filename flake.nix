{
  description = "shap — a shell-native interface for ACP-compatible coding agents";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # ACP adapters and coding agents, daily-built and cached upstream.
    # Intentionally not following our nixpkgs so we hit the upstream binary cache.
    llm-agents.url = "github:numtide/llm-agents.nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      llm-agents,
    }:
    let
      supportedSystems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      # Per-system package set with the Rust overlay applied.
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };

      # Single pinned toolchain (rust-toolchain.toml), shared by the dev shell
      # and the package build so what you test is what you ship.
      rustToolchainFor = system: (pkgsFor system).rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = rustToolchainFor system;
          agents = llm-agents.packages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [
              rustToolchain # cargo, rustc, clippy, rustfmt, rust-src
              pkgs.cargo-nextest
              pkgs.cargo-deny
              pkgs.rust-analyzer
              # Optional runtime helpers shap uses; absence never blocks build/test.
              pkgs.fzf
              pkgs.git
              # Optional ACP adapters to exercise the full flow.
              agents.codex-acp
              agents.claude-agent-acp
            ];

            env.RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = rustToolchainFor system;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
        in
        rec {
          shap = rustPlatform.buildRustPackage {
            pname = "shap";
            version = "0.1.0";
            src = pkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;

            # The commit tests shell out to `git`; make it available in the check phase.
            nativeCheckInputs = [ pkgs.git ];
          };
          default = shap;
        }
      );

      apps = forAllSystems (system: rec {
        shap = {
          type = "app";
          program = "${self.packages.${system}.shap}/bin/shap";
        };
        default = shap;
      });
    };
}
