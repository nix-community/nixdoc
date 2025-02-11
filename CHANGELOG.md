# Changelog

## Upcoming release

## Version 3.1.0

Add `--anchor-prefix` to remove or customize the `function-library-` prefix.

A header won't be rendered when `--description` and `--category` are empty.
This makes the generated markdown more flexible for inclusion in other documents.

## Version 3.0.8

Add `--json-output`, providing a JSON representation of the documentation.

Fix: attrsets in patterns are now skipped correctly.

## Version 3.0.7

Add support for empty prefix flags.
Allows for improved generic usage in nixpkgs/lib and other projects.

Empty prefix is now possible.
Issue: https://github.com/nix-community/nixdoc/issues/119 by @roberth

by @hsjobeki;

in https://github.com/nix-community/nixdoc/pull/122.

## Version 3.0.6

Exposes the package recipe under `recipes.default` so it can easily be re-used.
Example:

```
nix-repl> :l https://github.com/nixos/nixpkgs/tarball/nixpkgs-unstable
nix-repl> :l https://github.com/nix-community/nixdoc/tarball/master
nix-repl> pkgs.callPackage recipes.default {}
```

## Version 3.0.5

Fixes: incompatibility with nixpkgs in 3.0.3 and 3.0.4

by @hsjobeki;

in https://github.com/nix-community/nixdoc/pull/121.

## Version 3.0.4

Fixes: issue with headings ids introduced with 3.0.3

by @hsjobeki;

in https://github.com/nix-community/nixdoc/pull/117.

## Version 3.0.3

Fixes: shifting issue with commonmark headings https://github.com/nix-community/nixdoc/issues/113

by @hsjobeki;

in https://github.com/nix-community/nixdoc/pull/115.

## Version 3.0.2

Avoid displaying arguments when a doc-comment is already in place.

by @hsjobeki;

in https://github.com/nix-community/nixdoc/pull/109.

## Version 3.0.1

### New Features

- **Official Doc-Comments Support:** We've introduced support for official doc-comments as defined in [RFC145](https://github.com/NixOS/rfcs/pull/145). This enhancement aligns nixdoc with our latest documentation standard.

### Deprecated Features

- **Legacy Custom Format:** The custom nixdoc format is now considered a legacy feature. We plan to phase it out in future versions to streamline documentation practices.
- We encourage users to transition to the official doc-comment format introduced in this release.
- For now we will continue to maintain the legacy format, but will not accept new features or enhancements for it. This decision allows for a period of transition to the new documentation practices.

See [Migration guide](./doc/migration.md) for smooth transition

  by @hsjobeki; co-authored by @mightyiam

  in https://github.com/nix-community/nixdoc/pull/91.

## Version 3.0.0

Removed due to invalid lock file.

## 2.7.0

- Added support to customise the attribute set prefix, which was previously hardcoded to `lib`.
  The default is still `lib`, but you can pass `--prefix` now to use something else like `utils`.

  By @Janik-Haag in https://github.com/nix-community/nixdoc/pull/97

## 2.6.0

- After doing a great job of maintaining the project for this year, @asymmetric is passing on the torch to @infinisil!
- Multi-line comments at the top of the file now become the section description text.
  By @phaer in https://github.com/nix-community/nixdoc/pull/70

  For example, the following file
  ```nix
  /*
  This is just a test!
  */
  {
    /* Increments a number by one */
    increment = x: x + 1;
  }
  ```

  turns into the following markdown:

  ```markdown
  # Test {#sec-functions-library-test}
  This is just a test!

  ## `lib.test.increment` {#function-library-lib.test.increment}

  Increments a number by one

  `x`

  : Function argument
  ```

  whereas before, the top section would've been empty.

## 2.5.1

- readme: fix link to rendering example by @infinisil in https://github.com/nix-community/nixdoc/pull/67
- Fix indentation of structured multi-line comments by @asymmetric in https://github.com/nix-community/nixdoc/pull/81

## 2.5.0

## 2.4.0

- Fix line indentation stripping by @infinisil in https://github.com/nix-community/nixdoc/pull/62

## 2.3.0

- nix: remove outdated outputHashes by @asymmetric in https://github.com/nix-community/nixdoc/pull/38
- add snapshot testing by @asymmetric in https://github.com/nix-community/nixdoc/pull/39
- Create dependabot.yml by @asymmetric in https://github.com/nix-community/nixdoc/pull/41
- chore(deps): bump cachix/install-nix-action from 20 to 22 by @dependabot in https://github.com/nix-community/nixdoc/pull/42
- complete the markdown transition by @pennae in https://github.com/nix-community/nixdoc/pull/40

## 2.2.0

- Update rnix to 0.11 by @pennae [#36](https://github.com/nix-community/nixdoc/pull/36)

## 2.1.0

- Correctly support nested identifiers by @infinisil [#27](https://github.com/nix-community/nixdoc/pull/27)

## 2.0.0

- Switched output format from DocBook to CommonMark by @ryantm [#25](https://github.com/nix-community/nixdoc/pull/25)
