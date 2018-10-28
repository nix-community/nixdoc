{ pkgs ? import <nixpkgs> {} }:

(pkgs.callPackage ./Cargo.nix {}).nixdoc {}
