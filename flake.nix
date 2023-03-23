{
  description = "nixdoc";

  inputs = {
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-compat, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nixdoc";
          version = "1.0.1";
          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "arenatree-0.1.1" = "sha256-b3VVbYnWsjSjFMxvkfpJt13u+VC6baOIWD4qm1Gco4Q=";
              "rnix-0.4.1" = "sha256-C1L/qXk6AimH7COrBlqpUA3giftaOYm/qNxs7rQgETA=";
            };
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = "nixdoc";
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [ cargo clippy rustfmt ];
        };
      });
}
