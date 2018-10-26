{ pkgs ? import <nixpkgs> {} }:

# rnix requires the nightly Rust compiler.

let overlay = import (pkgs.fetchFromGitHub {
  owner  = "mozilla";
  repo   = "nixpkgs-mozilla";
  rev    = "c985206e160204707e15a45f0b9df4221359d21c";
  sha256 = "0k0p3nfzr3lfgp1bb52bqrbqjlyyiysf8lq2rnrmn759ijxy2qmq";
}) pkgs pkgs;

rust = {
  rustc = overlay.latest.rustChannels.nightly.rust;
  cargo = overlay.latest.rustChannels.nightly.rust;
};

buildRustPackage = pkgs.callPackage "${<nixpkgs>}/pkgs/build-support/rust" {
  inherit rust;
};

in buildRustPackage {
  name        = "nixdoc";
  version     = "0.1";
  cargoSha256 = "0z0lbl9r2b861w7lwndp0wf9w2w8q79bj5vzvzz27lg3sw1wm1jy";

  src = builtins.filterSource (f: t: baseNameOf f != ".git") ./.;

  # There are no tests currently! :sun:
  doCheck = false;
}
