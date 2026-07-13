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

## Export mode (`--manifest`)

Export mode takes a single `json`-file (*manifest*) that describes how all files need to be processed.
The result is one `export.json`

In contrast to `--json` and `--file` it produces data that is agnostic from the nixpkgs doc pipeline.
Making it suitable for any external consumer that has their own processing/rendering pipeline.

```sh
nixdoc --manifest manifest.json --root /path/to/nixpkgs --output functions-export.json
```

- `--manifest <FILE>`: manifest describing the sources to document (enables export mode)
- `--root <DIR>`: base directory (default: cwd)
- `--output <FILE>`: where to write the export JSON (default: stdout)

Only **RFC-145 `/**` doc-comments** are exported in this mode.

Each documented function is one entry: its `description` is the raw doc-comment prose.

### Manifest example

```json
{
  "version": 1,
  "groups": [
    { "id": "strings", "description": "string manipulation functions" }
  ],
  "sources": [
    { "path": "lib.strings", "file": "lib/strings.nix", "groups": ["strings"] },
    { "path": "pkgs.stdenv", "file": "pkgs/stdenv/generic/make-derivation.nix", "groups": ["stdenv"], "mode": "deep", "include": ["mkDerivation"] }
  ]
}
```

### Specification

#### Manifest schema (V1)

- `version` (int, required): manifest schema version. Currently `1`.
- `groups[]` (optional): Predefined groups registry.
  - `id` (string, required): group id, referenced by `sources[].groups`.
  - `description` (string, optional): **highest precedence**, overrides any [file-header]
    If not given is taken from the [file-header]

- `sources[]` (required) defined as:
  - `path` (string, required): dotted attribute-path prefix applied to every
    function found in `file`
  - `file` (string, required): relative path to a `.nix` file.
  - `groups` (string[], optional)
  - `mode` (string, optional): discovery strategy; defaults to `"flat"`.
    - `"flat"`: Extracts items at first top-level attrset.
    - `"deep"`: Search the whole file at any ast-depth. Must be acompanied by a non-empty `include`.
    - `"file"`: the file itself is an anonymous lambda expression. The file header is the lambda-doc
      `include`/`exclude` have no effect.
  - `include` (string[], optional): name allowlist applied **after** discovery.
  - `exclude` (string[], optional): name denylist applied **after** discovery.

Example of a `file-header` comment:

```nix
/**
  Library of foo
*/
{ ... }: {
  foo = x: x;
}
```

#### Export schema (V1)

```json
{
  "schemaVersion": 1,
  "groups": [
    { "id": "strings", "description": "string manipulation functions" }
  ],
  "entries": [
    {
      "id": "lib.strings.optionalString",
      "attrPath": "lib.strings.optionalString",
      "name": "optionalString",
      "description": "Depending on the boolean `cond` ... # Type ... # Example ...",
      "groups": ["strings"],
      "source": { "file": "lib/strings.nix", "line": 238, "column": 3 }
    }
  ]
}
```

- `schemaVersion` (int): export schema version.
- `groups[]`: resolved
  `description` (omitted when no description could be resolved). Sorted by `id`.
- `entries[]` list of:
  - `id` (string): URL safe id (' -> prime)
  - `attrPath` (string): dotted path, not URL safe.
  - `name` (string): function name.
  - `description` (string): doc-comment body as raw text.
  - `groups` (string[]): group ids this function belongs to, (sorted, de-duplicated).
  - `source`: `{ file (as given in the manifest), line (1-based), column (1-based) }`.

Entries are sorted by `id` ascending

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
