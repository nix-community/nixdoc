# Test for chained let ... in identifier resolution
# This tests the resolution of identifiers that point to other identifiers
# alias -> exports -> attrset
let
  /**
    Divides two numbers.

    # Arguments

    - a: The dividend
    - b: The divisor

    # Type

    ```
    divide :: Int -> Int -> Int
    ```
  */
  divide = a: b: a / b;

  # The actual exports attrset
  exports = {
    inherit divide;
  };

  # Alias points to exports
  alias = exports;

  # Another level of indirection
  doubleAlias = alias;
in
  # Return via chained identifier - nixdoc should resolve alias -> exports -> attrset
  alias
