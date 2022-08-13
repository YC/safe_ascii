use clap::Parser;
use safe_ascii::{map_to_escape, map_to_mnemonic, AsciiMapping};
use std::{
    env,
    fs::File,
    io::{self, BufReader, Write},
};

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Mode
    #[clap(
        short = 'm',
        long = "mode",
        value_name = "mnemonic|escape|suppress",
        value_parser,
        default_value = "mnemonic",
        takes_value = true,
        multiple_values = false,
        possible_values(&["mnemonic", "escape", "suppress"]),
        long_help = "mnemonic: abbreviation e.g. (NUL), (SP), (NL)
escape: \\x sequence, e.g. \\x00, \\x20, \\x0a
suppress: don't print non-printable characters"
    )]
    mode: String,

    /// Truncate
    #[clap(
        short = 't',
        long = "truncate",
        value_name = "truncate length",
        long_help = "length (bytes) to truncate at, -1 means no truncation",
        takes_value = true,
        multiple_values = false,
        default_value_t = -1
    )]
    truncate: i128,

    /// Exclude
    #[clap(
        short = 'x',
        long = "exclude",
        value_name = "exclude characters",
        value_delimiter = ',',
        long_help = "comma-delimited decimal values of characters to print
(9 is HT (tab), 10 is NL (newline), 13 is CR (carriage return), 32 is SP (space))",
        multiple_values = false,
        required = false,
        default_value = "10,32"
    )]
    exclude: Vec<String>,

    /// Files
    #[clap(
        name = "files",
        multiple_values = true,
        long_help = "A list of files to process.
Use '-' for stdin"
    )]
    files: Vec<String>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let mut truncate = args.truncate;
    let exclude = parse_exclude(args.exclude);

    let map_fn = match args.mode.as_str() {
        "mnemonic" => map_to_mnemonic,
        "escape" => map_to_escape,
        _ => |_| "".to_string(),
    };
    let mapping = AsciiMapping::new(&map_fn, exclude);

    // If files are given, then use files; otherwise, use stdin
    if !args.files.is_empty() {
        // files
        for filename in &args.files {
            if filename == "-" {
                try_process_file(io::stdin(), &mapping, &mut truncate)?;
            } else {
                let file = File::open(filename);
                match file {
                    Ok(file) => {
                        let buf_reader = BufReader::new(file);
                        try_process_file(buf_reader, &mapping, &mut truncate)?
                    }
                    Err(err) => {
                        eprintln!(
                            "{}: {}: {}",
                            env::args().next().expect("Cannot obtain executable name"),
                            filename,
                            err
                        );
                    }
                }
            }

            // Early return if no more chars should be printed
            if truncate == 0 {
                break;
            }
        }
    } else {
        try_process_file(std::io::stdin(), &mapping, &mut truncate)?
    }

    Ok(())
}

fn try_process_file<R: io::Read>(
    reader: R,
    mapping: &AsciiMapping,
    truncate: &mut i128,
) -> Result<(), std::io::Error> {
    if let Err(e) = process_file(reader, mapping, truncate) {
        if e.kind() == std::io::ErrorKind::BrokenPipe {
            std::process::exit(141);
        }
        Err(e)?
    };
    Ok(())
}

// Process files with Read trait
fn process_file<R: io::Read>(
    reader: R,
    mapping: &AsciiMapping,
    truncate: &mut i128,
) -> Result<(), std::io::Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for byte in reader.bytes() {
        match byte {
            Ok(c) => {
                if (33..=126).contains(&c) {
                    handle.write_all(&[c])?;
                } else {
                    handle.write_all(mapping.convert_u8(c).as_bytes())?;
                }

                // Reduce truncate
                if *truncate >= 0 {
                    *truncate -= 1;
                    if *truncate == 0 {
                        break;
                    }
                }
            }
            _ => break,
        }
    }
    Ok(())
}

// Parses exclude string
fn parse_exclude(s: Vec<String>) -> [bool; 256] {
    // Initialize to false
    let mut exclude: [bool; 256] = [false; 256];
    // Split by comma, parse into int, set index of exclude array
    for i in s {
        if let Ok(i) = str::parse::<u8>(&i) {
            exclude[i as usize] = true;
        }
    }
    exclude
}

#[cfg(test)]
mod cli {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Compares output of stdin and file inputs
    #[test]
    fn stdin_file() {
        // Adapted from cargo-script
        let target_dir =
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("target"));
        let program_path = format!("{}/debug/safe-ascii", target_dir);

        // ./safe-ascii -t 1000 ./safe-ascii
        let file_output = Command::new(&program_path)
            .args([program_path.as_str(), "-t", "1000"])
            .output()
            .unwrap();

        // ./safe-ascii -t 1000 < ./safe-ascii
        let file = std::fs::read(&program_path).unwrap();
        // https://stackoverflow.com/a/49597789
        let mut stdin_process = Command::new(&program_path)
            .args(["-t", "1000"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdin = stdin_process.stdin.as_mut().unwrap();
        stdin.write_all(&file[0..1000]).unwrap();
        let stdin_output = stdin_process.wait_with_output().unwrap();

        assert_eq!(&file_output.stdout, &stdin_output.stdout);
    }
}
