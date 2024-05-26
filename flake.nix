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
    let
      recipe = { lib, rustPlatform }:
        let
          package = (lib.importTOML ./Cargo.toml).package;
        in
        rustPlatform.buildRustPackage {
          pname = package.name;
          version = package.version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };
    in
    {
      recipes.default = recipe;
    }
    //
    flake-utils.lib.eachDefaultSystem (system:
      let
        nixpkgsDocs = import "${nixpkgs}/doc" { 
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ 
              (_: _: { nixdoc = self.packages.${system}.default; } )
            ];
          }; 
        };
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.callPackage recipe { };
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = self.packages.${system}.default.pname;
        };

        checks = {
          inherit nixpkgsDocs;
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
