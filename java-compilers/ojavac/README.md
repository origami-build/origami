# ojavac

ojavac is a simple wrapper for javac. Its interface is considerably cleaned up
compared to javac (which also means that it has fewer options for now, more will
be added when needed) and also provides some additional features that make it
usable when doing incremental compilation, namely the ability to output
dependency files, similar to gcc's -MT flag, that can then be used to infer
which classes need to be recompiled when a source file changes. In addition,
implicit compilation for class files is always disabled (-implicit:none for
javac).

## Usage

    USAGE:
        ojavac [OPTIONS] <source-file>...
    
    ARGS:
        <source-file>...    Java source file
    
    OPTIONS:
        -g, --debug                    Generate debugging information
        -h, --help                     Print help information
        -I, --include <path>           Include source file paths
        -l, --link <path>              Link against compiled JAR/class file paths
        -o, --out-dir <dir>            Specify output directory (default: package root)
            --package-root <dir>       Specify package root (default: auto-detected)
            --release <version>        Set Java SE release to compile against
        -V, --version                  Print version information
            --write-deps <file>        Write dependency manifest
            --write-makedeps <file>    Write dependency manifest in Make format
