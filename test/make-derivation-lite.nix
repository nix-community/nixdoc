/**
  Standard environment build helpers.
*/
# Minimal reproduction of nixpkgs' pkgs/stdenv/generic/make-derivation.nix
# includes 'overrideAttrs' and 'legacyThing' to test filtering.
# We only care about 'mkDerivation' in this example.
lib:
let
  inherit (lib) foldl';
in
config:
let
  # Some helper functions to make the code more readable.
  helper = x: x;
in
stdenv:
let
  /**
    This function creates a derivation, and returns it in the form of an
    attribute set that both looks like a derivation and can be built.

    # Arguments

    The single argument is an attribute set describing the package.

    # Example

    ```nix
    mkDerivation {
      name = "hello";
      src = ./.;
    }
    ```
  */
  mkDerivation =
    fnOrAttrs:
    fnOrAttrs;

  /**
    Override the arguments passed to `mkDerivation` bla bla
  */
  overrideAttrs =
    f:
    f;

  /* Code comment */
  legacyThing =
    y:
    y;
in
{
  inherit mkDerivation;
}
