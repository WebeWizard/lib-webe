Processes command line arguments.

Arguments have a long version (prefixed with 2 dashes):  
`--my-cool-carg \[optionalValue\]`  
Or a short version (prefixed with 1 dash):  
`-m \[optionalvalue\]`

The library also supports flags (an argument with no value)

NOTE:  The resulting Arg object will only capture arguments defined by valid ArgOpts.  You can still inspect env::args() if you need to access some un-defined argument.