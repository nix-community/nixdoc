{
  # not a doc comment
  hidden = a: a;

  /* 
  nixdoc-legacy comment 
  
  Example:

  This is a parsed example

  Type:

  This is a parsed type
  */
  nixdoc = {};

  /**
  doc comment in markdown format
  */
  rfc-style = {};

  # Omitting a doc comment from an attribute doesn't duplicate the previous one 
  /** Comment */
  foo = 0;

  # This should not have any docs
  bar = 1;
  
}
