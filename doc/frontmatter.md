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
    tags:
      - foo
      - bar
    ---
  */
  foo = x: x;
}
```

In this example `key` is `tags` and `value` is `List("foo", "bar")`.

See also: [yaml specification](https://yaml.org/spec/1.2.2/) for how to use YAML.

## Keywords

### `doc-location`

Rules:

1. The `doc-location` keyword can be used to use content from another file INSTEAD of the doc-comments content.

2. The value `file` must be given as a path relative to the current file.

3. Using this directive does not process any content inside the file, using it as-is.

```nix
{
  /** 
  ---
  doc-location: ./path.md
  ---
  */
  foo = x: x;
}
```

`path.md`
```md
some nice docs!
```

In this example, the `doc-location` directive fetches content from `./path.md` treating it as if it were directly within the doc-comment.
This allows tracking the reference between the source position and the markdown file, in case of external documentation.

## Extensibility

The initial set of keywords is intentionally minimalistic, focusing on immediate and broadly applicable needs.

Community contributions are encouraged to expand this list as new use cases emerge.

## Error handling

Any issues encountered during the processing of frontmatter—be it syntax errors, invalid paths, or unsupported keywords—should result in clear, actionable error messages to the user.

## Future work

This proposal represents a foundational step towards more versatile and extendable reference documentation.
As we move forward, we'll remain open to adapting and expanding this specification to meet emerging needs and leverage community insights.
