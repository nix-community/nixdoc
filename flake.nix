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
      let
        pkgs = nixpkgs.legacyPackages.${system};
        package = (pkgs.lib.importTOML ./Cargo.toml).package;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = package.name;
          version = package.version;
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = package.name;
        };

        checks = {
          test = self.packages.${system}.default;

          clippy = self.packages.${system}.default.overrideAttrs (_: {
            checkPhase = ''
              ${pkgs.clippy}/bin/cargo-clippy
            '';
          });
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            cargo-insta
            clippy
            rustfmt
          ];
        };
      });
}
