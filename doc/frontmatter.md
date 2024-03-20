# Frontmatter

This document is the specification for custom metadata within doc-comments.

It should only apply to doc-comments (`/** */`) and not to regular code comments to ensure a limited scope.

## Why frontmatter is needed (sometimes)

Sometimes it is desireable to extend the native doc-comment functionality. For that user scenario, frontmatter can be optionally used.

i.e.,

- Enriching documentation generation.
- Maintaining References.
- Metadata inclusion.
- Allow better Handling of edge cases.

## Detailed design

Fields (from Keywords list) can be defined in frontmatter.

Frontmatter is defined using key-value pairs, encapsulated within triple-dashed lines (---).

While there is no strict specification for frontmatter formats, YAML is commonly preferred. Although JSON could also be used alternatively.

`{key}` is a placeholder for the list of available keywords listed below.

`{value}` is a placeholder for the set value associated with the directive.

Example:

```nix
{
/** 
  ---
  import: ./path.md
  ---
*/
foo = x: x;
}
```

In this example, the `import` directive fetches content from `./path.md` treating it as if it were directly within the doc-comment.
This allows tracking the reference between the source position and the markdown file, in case of extensive documentation.

## Keywords

### Import

Rules:

1. The `import` keyword can be used to use content from another file INSTEAD of the doc-comments content.

2. The value `file` must be given as an absolute path (relative to git root) or relative to the current file.

3. There can only be one `import` per doc-comment.

```nix
{
/** 
 ---
 import: ./path.md
 ---
*/
foo = x: x;
}
```

`path.md`
```md
some nice docs!
```

Rendering this would behave as if the content where actually placed in the doc-comment itself.

Placing frontmatter inside the imported file will be ignored. (No nested directives)
Since handling recursive or nested imports adds too much complexity for little or no benefit.

> Note: Absolute path imports are relative to the repository root. They only work inside `git` repositories and require having the `git` binary in PATH.
> This is most commonly used in large repositories or when the documentation files are not placed alongside the .nix files.

## Extensibility

The initial set of keywords is intentionally minimalistic,
focusing on immediate and broadly applicable needs.

Community contributions are encouraged to expand this list as new use cases emerge.

## Error handling

Any issues encountered during the processing of frontmatter—be it syntax errors, invalid paths, or unsupported keywords—should result in clear, actionable error messages to the user.

## Future work

This proposal represents a foundational step towards more versatile and extendable reference documentation.
As we move forward, we'll remain open to adapting and expanding this specification to meet emerging needs and leverage community insights.
