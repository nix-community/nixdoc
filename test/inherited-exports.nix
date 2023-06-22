{ lib }:

let
  /* Concatenate a list of strings.

    Type: concatStrings :: [string] -> string

     Example:
       concatStrings ["foo" "bar"]
       => "foobar"
  */
  concatStrings = builtins.concatStringsSep "";

  /* this should be ignored because it's inherited from an explicit source */
  from = a: a;
in {
  inherit concatStrings;
  inherit ({}) from;

  foo1 = {
    /* this should be ignored because it's in a nested attrset */
    bar = a: a;
  };

  /* this should be found */
  foo2 =
    let
      /* this should be ignored because it's in a nested let */
      bar = a: a;
    in bar;
}
