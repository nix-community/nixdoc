/**
  String manipulation functions.
*/
{ lib }:
let
  inherit (builtins) length;
in
rec {
  /**
    Depending on the boolean `cond`, return either the given string
    or the empty string. Useful to concatenate against a bigger string.

    # Type

    ```
    optionalString :: bool -> string -> string
    ```

    # Example

    ```nix
    optionalString true "some-string"
    => "some-string"
    ```
  */
  optionalString =
    cond:
    string: if cond then string else "";

  /**
    Like `mapAttrs`, but the function additionally receives the position.

    # Example

    ```nix
    mapAttrs' (name: value: nameValuePair name value) { a = 1; }
    ```
  */
  mapAttrs' =
    f:
    set: builtins.listToAttrs (map (attr: f attr set.${attr}) (builtins.attrNames set));

  /* Legacy-only doc comment. */
  concatStrings = builtins.concatStringsSep "";

  # A plain line comment
  undocumented = x: x;
}
