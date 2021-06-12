nixdoc
======

This tool is (for now) a proof-of-concept to generate documentation
for Nix library functions from the source files in `nixpkgs/lib`.

It uses [rnix][] to parse Nix source files, which are then transformed
into a CommonMark (with some syntax extensions) representation of the
function set.

Please see [this Discourse thread][] for information on the
documentation format and general discussion.

Check out [this example][] of documentation generated for the
`strings.nix` file.

## Comment format

(Note: The parser for this is a quick hack, I don't want to spend time
writing a better one before I know how it's supposed to work.)

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

Please check the [issues][] page.

## Building

This project requires a nightly Rust compiler build.

[rnix]: https://gitlab.com/jD91mZM2/rnix
[this Discourse thread]: https://discourse.nixos.org/t/nixpkgs-library-function-documentation-doc-tests/1156
[this example]: https://storage.googleapis.com/files.tazj.in/nixdoc/manual.html#sec-functions-library-strings
[issues]: https://github.com/nix-community/nixdoc/issues
