# Migration Guide

Upgrading from nixdoc <= 2.x.x to >= 3.0.0

To leverage the new doc-comment features and prepare for the deprecation of the legacy format, follow these guidelines:

## Documentation Comments

- Use double asterisks `/** */` to mark comments intended as documentation. This differentiates them from internal comments and ensures they are properly processed as part of the documentation.

**Example:**

`lib/attrsets.nix (old format)`
````nix
/* Filter an attribute set by removing all attributes for which the
   given predicate return false.
   Example:
     filterAttrs (n: v: n == "foo") { foo = 1; bar = 2; }
     => { foo = 1; }
   Type:
     filterAttrs :: (String -> Any -> Bool) -> AttrSet -> AttrSet
*/
filterAttrs =
  # Predicate taking an attribute name and an attribute value, which returns `true` to include the attribute or `false` to exclude the attribute.
  pred:
  # The attribute set to filter
  set:
  listToAttrs (concatMap (name: let v = set.${name}; in if pred name v then [(nameValuePair name v)] else []) (attrNames set));
````

->

`lib/attrsets.nix (new format)`
````nix
/**
  Filter an attribute set by removing all attributes for which the
  given predicate return false.
  
  # Example

  ```nix
  filterAttrs (n: v: n == "foo") { foo = 1; bar = 2; }
  => { foo = 1; }
  ```

  # Type
  
  ```
  filterAttrs :: (String -> Any -> Bool) -> AttrSet -> AttrSet
  ```
  
  # Arguments

  **pred**
  : Predicate taking an attribute name and an attribute value, which returns `true` to include the attribute, or `false` to exclude the attribute.
  
  **set**
  : The attribute set to filter
*/
filterAttrs =
  pred:
  set:
  listToAttrs (concatMap (name: let v = set.${name}; in if pred name v then [(nameValuePair name v)] else []) (attrNames set));
````

## Documenting Arguments

With the introduction of RFC145, there is a shift in how arguments are documented. While direct "argument" documentation is not specified, you can still document arguments effectively within your doc-comments by writing explicit markdown.

**Example:** Migrating **Single Argument Documentation**

The approach to documenting single arguments has evolved. Instead of individual argument comments, document the function and its arguments together.

> Note: Within nixpkgs the convention of using [definition-lists](https://www.markdownguide.org/extended-syntax/#definition-lists) for documenting arguments has been established.

```nix
{
  /**
  The `id` function returns the provided value unchanged.
  
  # Arguments
  
  `x` (Any)
  : The value to be returned.
  
  */
  id = x: x;
}
```

If arguments require more complex documentation consider starting an extra section per argument

```nix
{
  /**
  The `id` function returns the provided value unchanged.
  
  # Arguments
  
  ## **x** (Any)
  (...Some comprehensive documentation)

  */
  id = x: x;
}
```

**Example:** Documenting Structured Arguments
Structured arguments can be documented (described in RFC145 as 'lambda formals'), using doc-comments.

```nix
{
  /**
  The `add` function calculates the sum of `a` and `b`.
  */
  add = { 
      /** The first number to add. */
      a, 
      /** The second number to add. */
      b 
    }: a + b;
}
```

Ensure your documentation comments start with double asterisks to comply with the new standard. The legacy format remains supported for now but will not receive new features. It will be removed once important downstream projects have been migrated.
