{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rainix.url = "github:rainprotocol/rainix";
  };

  outputs = { self, flake-utils, rainix }:

  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = rainix.pkgs.${system};

      # cargo-expand is a bin used with macrotest crate for testing macros
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
          repo = pname;
          tag = version;
          hash = "sha256-VEjgSmZcy/CZ8EO/mJ2nBOpQviF4A/QQ8SpLLF/9x4c=";
        };
        useFetchCargoVendor = true;
        cargoHash = "sha256-ow5Zy0tv9W5w+Pib2yW1nPj2pUZt0HhplHxjIZZZzU8=";
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
            CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER='wasm-bindgen-test-runner' cargo test --target wasm32-unknown-unknown --workspace
          '';
        };

        rainix-rs-test = rainix.mkTask.${system} {
          name = "rainix-rs-test";
          body = ''
            set -euxo pipefail
            cargo test --workspace
          '';
        };

        rainix-rs-artifacts = rainix.mkTask.${system} {
          name = "rainix-rs-artifacts";
          body = ''
            set -euxo pipefail
            cargo build --release --workspace
          '';
        };
      };

      # For `nix develop`:
      devShells.default = pkgs.mkShell {
        packages = [
          packages.rainix-wasm-artifacts
          packages.rainix-wasm-test
          packages.rainix-rs-test
          packages.rainix-rs-artifacts
          cargo-expand
        ];
        buildInputs = rainix.devShells.${system}.default.buildInputs;
        nativeBuildInputs = rainix.devShells.${system}.default.nativeBuildInputs;
      };
    }
  );
}
