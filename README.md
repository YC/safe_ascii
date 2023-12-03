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
          Use '-' for stdin.

Options:
  -m, --mode <mnemonic|escape|suppress>
          Mode of character conversion/suppression.

          [default: mnemonic]

          Possible values:
          - mnemonic: Abbreviation, e.g. (NUL), (SP), (NL)
          - escape:   Hex sequence, e.g. \x00, \x20, \x0a
          - suppress: Suppress non-printable characters

  -t, --truncate <truncate-length>
          Length (bytes) to truncate at, -1 represents no truncation.

          [default: -1]

  -x, --exclude <exclude-characters>
          Comma-delimited decimal values of characters to print.
          (9 is HT (tab), 10 is NL (newline), 13 is CR (carriage return), 32 is SP (space))

          [default: 10,32]
```

### Example

```
$ safe-ascii -x 10 Cargo.toml
[package]
name(SP)=(SP)"safe-ascii"
version(SP)=(SP)"1.0.0"
...
```
