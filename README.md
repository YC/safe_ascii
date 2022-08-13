# safe_ascii
A tool for sanitising files to printable ASCII characters.

Can be used when marking programming projects, where the output may be incorrect and it may be ideal to avoid non-printable characters (e.g. to avoid file being detected as binary when using `grep`).

## Features
- stdin
- Substitution of non-printable characters, with modes: mnemonic (e.g. (NUL)), escape sequence (e.g. \x0), or suppress
- Truncation

## Usage
```
USAGE:
    safe-ascii [OPTIONS] [files]...

ARGS:
    <files>...
            A list of files to process.
            Use '-' for stdin

OPTIONS:
    -h, --help
            Print help information

    -m, --mode <mnemonic|escape|suppress>
            mnemonic: abbreviation e.g. (NUL), (SP), (NL)
            escape: \x sequence, e.g. \x00, \x20, \x0a
            suppress: don't print non-printable characters[default: mnemonic]
            [possible values: mnemonic, escape, suppress]

    -t, --truncate <truncate length>
            length (bytes) to truncate at, -1 means no truncation[default: -1]

    -V, --version
            Print version information

    -x, --exclude <exclude characters>
            comma-delimited decimal values of characters to print
            (9 is HT (tab), 10 is NL (newline), 13 is CR (carriage return), 32 is SP
            (space))[default: 10,32]
```

### Example

```
$ safe-ascii -x 10 Cargo.toml
[package]
name(SP)=(SP)"safe_ascii"
version(SP)=(SP)"1.0.0"
edition(SP)=(SP)"2018"
...
```
