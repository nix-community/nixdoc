{
  /* nixdoc comment */
  old = 
  # Should be visible
  arg: 1;
  
  /** Doc-comment */
  omited = 
  # Not visible
  arg: 1; 

  /** Doc-comment */
  multiple = 
  # Not visible
  arg:
  /* Not visible */ 
  foo:
  /** Not visible */
  bar:
  1;

  /** 
    Doc-comment before the lamdba causes the whole 
    lambda including its arguments to switch to doc-comments ONLY rendering
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
    Not shown yet
    */
    formal4,

  }: 
  {};

  /*
    Legacy comments allow to use any 
    form of comments for the lambda arguments/formals
  */
  legacyArgumentTest = {
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
    doc-comment style
    */
    formal4,

  }: 
  {};
}