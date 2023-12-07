# Changelog

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
