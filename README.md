# nixdoc

This tool is used to generate reference documentation for Nix library functions defined in [Nixpkgs' `lib`](https://github.com/NixOS/nixpkgs/tree/master/lib).

Check out [this example](https://nixos.org/manual/nixpkgs/unstable/#sec-functions-library-strings) of documentation generated for the [`lib/strings.nix`](https://github.com/NixOS/nixpkgs/blob/nixpkgs-unstable/lib/strings.nix) file.

It uses [rnix](https://github.com/nix-community/rnix-parser) to parse Nix source files,
which are then transformed into a CommonMark (with some syntax extensions) representation of the
function set.

## Comment format

Currently, identifiers are included in the documentation if they have
a preceding comment in multiline syntax `/* something */`.

Two special line beginnings are recognised:

* `Example:` Everything following this line will be assumed to be a
  verbatim usage example.
* `Type:` This line will be interpreted as a faux type signature.

These will result in appropriate elements being inserted into the
output.

## Function arguments

Function arguments can be documented by prefixing them with a comment:

```
/* This function does the thing a number of times. */
myFunction =
    # The thing to do
    thing:
    # How many times to do it
    n: doNTimes n thing
```

## Caveats & TODOs

Please check the [issues](https://github.com/nix-community/nixdoc/issues) page.
