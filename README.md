# nixdoc

This tool is used to generate reference documentation for Nix library functions defined in [Nixpkgs' `lib`](https://github.com/NixOS/nixpkgs/tree/master/lib).

Check out [this example](https://nixos.org/manual/nixpkgs/unstable/#sec-functions-library-strings) of documentation generated for the [`lib/strings.nix`](https://github.com/NixOS/nixpkgs/blob/nixpkgs-unstable/lib/strings.nix) file.

It uses [rnix](https://github.com/nix-community/rnix-parser) to parse Nix source files,
which are then transformed into a CommonMark (with some syntax extensions) representation of the
function set.

## Comment format

This tool implements a subset of the doc-comment standard specified in [RFC-145/doc-comments](https://github.com/NixOS/rfcs/blob/master/rfcs/0145-doc-strings.md).
But, it is currently limited to generating documentation for statically analysable attribute paths only.
In the future, it could be the role of a Nix interpreter to obtain the values to be documented and their doc-comments.

It is important to start doc-comments with the additional asterisk (`*`) -> `/**` which renders as a doc-comment.

The content of the doc-comment should conform to the [Commonmark](https://spec.commonmark.org/0.30/) specification.

### Example

The following is an example of markdown documentation for new and current users of nixdoc.

> Sidenote: Indentation is automatically detected and should be consistent across the content. 
> 
> If you are used to multiline-strings (`''`) in nix this should be intuitive to follow.

````nix
{
  /** 
    This function adds two numbers

    # Example

    ```nix
    add 4 5
    =>
    9
    ```

    # Type

    ```
    add :: Number -> Number -> Number
    ```

    # Arguments

    a
    : The first number
    
    b
    : The second number
    
  */
  add = a: b: a + b;
}
````

> Note: Within nixpkgs the convention of using [definition-lists](https://www.markdownguide.org/extended-syntax/#definition-lists) for documenting arguments has been established.

## Usage

Refer to `nixdoc --help` for the most up-to-date usage information.

For a minimal format, suitable for inclusion into a dedicated documentation page, use:

```sh
nixdoc --file lib.nix --category "" --description "" --prefix "" --anchor-prefix "" >lib.md
```

## Custom nixdoc format (Legacy)

You should consider migrating to the newer format described above.

See [Migration guide](./doc/migration.md).

### Comment format (legacy)

Identifiers are included in the documentation if they have
a preceding comment in multiline syntax `/* something */`. You should consider migrating to the new format described above.

Two special line beginnings are recognized:

* `Example:` Everything following this line will be assumed to be a
  verbatim usage example.
* `Type:` This line will be interpreted as a faux-type signature.

These will result in appropriate elements being inserted into the
output.

### Function arguments (legacy)

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
