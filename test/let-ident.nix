# Test for let ... in identifier pattern
# This tests the resolution of identifiers that refer to attrsets defined in let bindings
let
  /**
    Adds two numbers together.

    # Arguments

    - a: The first number
    - b: The second number

    # Type

    ```
    add :: Int -> Int -> Int
    ```

    # Example

    ```nix
    add 1 2
    => 3
    ```
  */
  add = a: b: a + b;

  /**
    Multiplies two numbers.

    # Arguments

    - x: The first number
    - y: The second number

    # Type

    ```
    multiply :: Int -> Int -> Int
    ```
  */
  multiply = x: y: x * y;

  # This is not documented (no doc comment)
  undocumented = x: x;

  # The actual exports attrset
  exports = {
    inherit add multiply;
  };

  # Test chained resolution: alias points to exports
  alias = exports;
in
  # Return via identifier - nixdoc should resolve this to the exports attrset
  exports
