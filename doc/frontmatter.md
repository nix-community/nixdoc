# Frontmatter

This document is the specification for custom metadata within doc-comments.

It should only apply to doc-comments (`/** */`) and not to regular code comments to ensure a limited scope.

## Why frontmatter is needed (sometimes)

Sometimes it is desireable to extend the native doc-comment functionality. For that user-scenario, frontmatter can be optionally used.

Frontmatter is the de-facto standard of adding document specific meta tags to markdown. [1](https://docs.github.com/en/contributing/writing-for-github-docs/using-yaml-frontmatter) [2](https://jekyllrb.com/docs/front-matter/) [3](https://starlight.astro.build/reference/frontmatter/)

it can be useful for:

- Enriching documentation generation.
- Maintaining References.
- Metadata inclusion.
- Allow better Handling of documentation edge cases.

## Detailed design

Frontmatter is defined using `key`-`value` pairs, encapsulated within triple-dashed lines (---).

While there is no strict specification for frontmatter formats, YAML is commonly preferred.
Although JSON could also be used alternatively.

Only `key`s from the list of available keywords can be used.

The `value` is any valid yaml value. Its type and semantics is specified by each keyword individually.

Example:

```nix
{
  /** 
    ---
    key: value
    ---
  */
  foo = x: x;
}
```

See also: [yaml specification](https://yaml.org/spec/1.2.2/) for how to use YAML.

## Keywords

### `doc_location`

Rules:

1. The `doc_location` keyword can be used to use content from another file and **forbid** any further content to be added to the existing doc-comment.

2. The value must be given as a path relative to the current file.

3. The file pointed to by `doc_location` will be used as-is, without any further processing done to it.

```nix
{
  /** 
  ---
  doc_location: ./path.md
  ---
  */
  foo = x: x;
}
```

`path.md`
```md
some nice docs!
```

In this example, the `doc_location` directive fetches content from `./path.md` and treats it as the actual doc-comment.
This allows tracking the reference between the source position and the markdown file, in case of external documentation.

#### Design decision: Relative vs Absolute paths

This section explains the decision to only use relative paths and not support absolute paths for the file pointed to by `doc_location`.

<details>
<summary>Should absolute paths be allowed?</summary>

- (+) When the docs are entirely elsewhere, e.g. `doc/manual/..`, a relative path would have to be `../../..`, very ugly
  - (-) If only relative paths are allowed, encourages moving docs closer to the source, makes changing documentation easier.
    - For the nix-build, adjustments of which files are included in the derivation source may be needed.
  - (-) With only relative paths, it's more similar to NixOS module docs
- (-) We can still allow absolute paths later on if necessary
- (-) Makes it very confusing where absolute paths are relative to (build root, git root, `.nix` location, etc.)
  - (+) Could use a syntax like `$GIT_ROOT/foo/bar`
    - (-) Relies on a Git repository and git installed
    - (-) Not a fan of more custom syntax

**Decision**: Not supported by now.

This outcome was discussed in the nix documentation team meeting: https://discourse.nixos.org/t/2024-03-21-documentation-team-meeting-notes-114/41957.

</details>

## Error handling

Any issues encountered during the processing of frontmatter — be it syntax errors, invalid paths, or unsupported keywords—should result in clear, actionable error messages to the user.

## Extensibility

The initial set of keywords is intentionally minimalistic, focusing on immediate and broadly applicable needs.

When extending this document we ask our contributors to be tooling agnostic, such that documentation wont't rely on any implementation details.
This approach ensures that the documentation remains independent of specific implementation details. By adhering to this principle, we aim to create a resource that is universally applicable.

## Future work

This proposal represents a foundational step towards more versatile and extendable reference documentation.
As we move forward, we'll remain open to adapting and expanding this specification to meet emerging needs and leverage community insights.
