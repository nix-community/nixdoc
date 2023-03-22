{
  description = "nixdoc";

  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system}; in
      {
        packages.default = (pkgs.callPackage ./Cargo.nix { }).nixdoc { };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
          name = "nixdoc";
        };
      });
}
