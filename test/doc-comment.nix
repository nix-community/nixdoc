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

  /**
  doc comment in markdown format

  # Example (Should be a heading)

  This is just markdown

  Type: (Should NOT be a heading)

  This is just markdown
  */
  argumentTest = {
    # Legacy line comment
    formal1,
    # Legacy 
    # Block
    formal2,
    /*
    Legacy 
    multiline
    comment
    */
    formal3,
    /**
    official doc-comment variant
    */
    formal4,

  }: 
  {};

  # Omitting a doc comment from an attribute doesn't duplicate the previous one 
  /** Comment */
  foo = 0;

  # This should not have any docs
  bar = 1;
  
}
