{
  description = "shap — a shell-native interface for ACP-compatible coding agents";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Crane has no flake inputs of its own; it picks up our nixpkgs via mkLib.
    crane.url = "github:ipetkov/crane";

    # ACP adapters and coding agents, daily-built and cached upstream.
    # Intentionally not following our nixpkgs so we hit the upstream binary cache.
    llm-agents.url = "github:numtide/llm-agents.nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
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

      # Crane builds dependencies in a separate derivation (buildDepsOnly),
      # so source-only changes reuse the compiled dependency artifacts
      # instead of rebuilding every crate from scratch.
      craneOutputsFor =
        system:
        let
          pkgs = pkgsFor system;
          craneLib = (crane.mkLib pkgs).overrideToolchain (rustToolchainFor system);

          # Cargo sources plus insta `.snap` snapshot files, which the test
          # suite reads during the check phase.
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              (craneLib.filterCargoSources path type) || (pkgs.lib.hasSuffix ".snap" path);
            name = "source";
          };

          commonArgs = {
            pname = "shap";
            version = "0.1.0";
            inherit src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          shap = craneLib.buildPackage (
            commonArgs
            // {
              inherit cargoArtifacts;
              # The commit tests shell out to `git`; make it available in the check phase.
              nativeCheckInputs = [ pkgs.git ];
            }
          );

          clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
        };
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
        system: rec {
          shap = (craneOutputsFor system).shap;
          default = shap;
        }
      );

      checks = forAllSystems (
        system: {
          inherit (craneOutputsFor system) shap clippy;
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
