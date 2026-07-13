let
  /**
    Doc L3:C3
  */
  fun = x: x;
  #^ Position should point to this line, not the inherit below.
in
{
  inherit fun;
}
