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
        packages.default = (pkgs.callPackage ./Cargo.nix { }).nixdoc { };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = "nixdoc";
        };

        devShells.default = pkgs.mkShellNoCC {
          buildInputs = with pkgs; [ cargo rustfmt ];
        };
      });
}
