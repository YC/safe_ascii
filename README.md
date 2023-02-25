# safe_ascii
A tool for sanitising input to printable ASCII characters.

Used for marking programming assignments, where the output may contain
unexpected binary characters which taint the test execution output.

## Usage
```
Usage: safe-ascii [OPTIONS] [files]...

Arguments:
  [files]...
          A list of files to process.
          Use '-' for stdin

Options:
  -m, --mode <mnemonic|escape|suppress>
          mnemonic: abbreviation e.g. (NUL), (SP), (NL)
          escape: \x sequence, e.g. \x00, \x20, \x0a
          suppress: don't print non-printable characters

          [default: mnemonic]
          [possible values: mnemonic, escape, suppress]

  -t, --truncate <truncate-length>
          length (bytes) to truncate at, -1 means no truncation

          [default: -1]

  -x, --exclude <exclude-characters>
          comma-delimited decimal values of characters to print
          (9 is HT (tab), 10 is NL (newline), 13 is CR (carriage return), 32 is SP (space))

          [default: 10,32]

  -h, --help
          Print help information (use `-h` for a summary)

  -V, --version
          Print version information
```

### Example

```
$ safe-ascii -x 10 Cargo.toml
[package]
name(SP)=(SP)"safe-ascii"
version(SP)=(SP)"1.0.0"
...
```
