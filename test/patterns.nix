{ 
  pkgs ? import <nixpkgs>
    # | This attrset must be skipped -- it is not the top-level attrset
    # ↓ even though it comes first!
    {   }
}:

{
  /** 
    This binding is in the top-level attrset
  */
  iAmTopLevel = null;
}
