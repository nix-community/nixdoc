/**
Prequel
*/
{lib}:
{

   /**
     Create a file set from a path that may or may not exist
   */
   packagesFromDirectoryRecursive =
     # Options.
     {
       /**
         rfc style

         ```
         Path -> AttrSet -> a
         ```
       */
       callPackage,
       /*
         legacy multiline

         ```
         Path
         ```
       */
       directory,
       #  legacy single line
       config,
       # legacy
       # block 
       # comment
       moreConfig,
     }:
   1;
}