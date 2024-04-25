# h1-heading

## h2-heading

## h2-heading-with-id {#some-id}

### h3-heading

```nix
# A comment should not be shifted
```

### annother heading

```
# Some pseudocode
map a from b -> 1
```

### Some heading

````nix
/**
   ```nix
   # A nested comment should not be shifted
   ```
*/
1
# A comment
````

#### h4-heading

Nested tilde fences

~~~~~nix
/*
   ~~~~nix
      /**
         ~~~nix
            # A nested comment should not be shifted
            42
         ~~~
      */
      1
      # A nested comment ^
   ~~~~
*/
# A comment ^
foo
~~~~~


##### h5-heading

Mixed fences

~~~nix
/**
   ```nix
   # A nested comment should not be shifted
   ```
*/
1
# A comment
~~~

###### h6-heading

This should be h6 as well
