{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rainix.url = "github:rainprotocol/rainix";
  };

  outputs = { self, flake-utils, rainix }:

  flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = rainix.pkgs.${system};
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
        ];
        buildInputs = rainix.devShells.${system}.default.buildInputs;
        nativeBuildInputs = rainix.devShells.${system}.default.nativeBuildInputs;
      };
    }
  );
}