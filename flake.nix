{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rainix.url = "github:rainprotocol/rainix";
  };

  outputs = { self, flake-utils, rainix }:

  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = rainix.pkgs.${system};

      # need to build from source since it errors on macos with current rainix rust version 1.79
      # and the version available on rainix.pkgs is 1.0.100 which is not compatible with rust 1.79,
      # the latest version that works with rust 1.79 is v1.0.95 so we build form source
      cargo-expand = (pkgs.makeRustPlatform{
        rustc = rainix.rust-toolchain.${system};
        cargo = rainix.rust-toolchain.${system};
      }).buildRustPackage rec {
        pname = "cargo-expand";
        version = "1.0.95";
        src = pkgs.fetchFromGitHub {
          executable = true;
          owner = "dtolnay";
          repo = "cargo-expand";
          tag = "1.0.95";
          hash = "sha256-VEjgSmZcy/CZ8EO/mJ2nBOpQviF4A/QQ8SpLLF/9x4c=";
        };
        cargoHash = "sha256-m/F6fI1d8i5lVyURti86FWAs/U14TXpgg/nemLAv4NI=";
      };
    in rec {
      packages = rec {

        rainix-wasm-artifacts = rainix.mkTask.${system} {
          name = "rainix-wasm-artifacts";
          body = ''
            set -euxo pipefail
            cargo build -r --target wasm32-unknown-unknown
          '';
        };

        rainix-wasm-test = rainix.mkTask.${system} {
          name = "rainix-wasm-test";
          body = ''
            set -euxo pipefail
            CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='wasm-bindgen-test-runner' cargo test --target wasm32-unknown-unknown
          '';
        };
      };

      # For `nix develop`:
      devShells.default = pkgs.mkShell {
        packages = [
          packages.rainix-wasm-artifacts
          packages.rainix-wasm-test
          cargo-expand
        ];
        buildInputs = rainix.devShells.${system}.default.buildInputs;
        nativeBuildInputs = rainix.devShells.${system}.default.nativeBuildInputs;
      };
    }
  );
}
