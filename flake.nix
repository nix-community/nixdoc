{
  description = "nixdoc";

  inputs = {
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = { self, nixpkgs, flake-utils, ... }:
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
          test = self.packages.${system}.default.overrideAttrs (drvAttrs: {
            postCheck = drvAttrs.postCheck or "" + ''
              ${pkgs.rustfmt}/bin/rustfmt --check src/**.rs
              ${pkgs.clippy}/bin/cargo-clippy --no-deps -- -D warnings
            '';
          });
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            cargo-insta
            clippy
            rustfmt
            rustc
          ];
        };
      });
}
