#![warn(clippy::pedantic)]

use clap::Parser;
use safe_ascii::{AsciiMapping, map_suppress, map_to_escape, map_to_mnemonic};
use std::{
    env, error,
    fs::File,
    io::{self, BufReader, Read, Write},
    process,
};

/// Mode of conversion/suppression.
#[derive(clap::ValueEnum, Clone)]
enum Mode {
    /// Abbreviation, e.g. (NUL), (SP), (LF)
    Mnemonic,
    /// Hex sequence, e.g. \x00, \x20, \x0a
    Escape,
    /// Suppress non-printable characters
    Suppress,
}

/// CLI Definition for clap
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Mode
    #[arg(
        value_enum,
        short = 'm',
        long = "mode",
        value_name = "mnemonic|escape|suppress",
        default_value = "mnemonic",
        num_args(1),
        long_help = "Mode of character conversion/suppression."
    )]
    mode: Mode,

    /// Truncate
    #[arg(
        short = 't',
        long = "truncate",
        value_name = "truncate-length",
        long_help = "Length (bytes) to truncate at, -1 represents no truncation.",
        num_args(1),
        default_value_t = -1
    )]
    truncate: i128,

    /// Exclude
    #[arg(
        short = 'x',
        long = "exclude",
        value_name = "exclude-characters",
        value_delimiter = ',',
        long_help = "Comma-delimited decimal values of non-printable characters to print (empty string for none).
(9 is HT (tab), 10 is NL (newline), 13 is CR (carriage return), 32 is SP (space))",
        num_args(1),
        required = false,
        default_value = "10,32"
    )]
    exclude: Vec<String>,

    /// Files
    #[arg(
        value_name = "files",
        num_args(0..),
        long_help = "A list of files to process.
Use '-' for stdin."
    )]
    files: Vec<String>,
}

fn main() -> Result<(), io::Error> {
    let args = Args::parse();
    let exclude = match parse_exclude(args.exclude) {
        Ok(exclude) => exclude,
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    };

    let map_fn = match args.mode {
        Mode::Mnemonic => map_to_mnemonic,
        Mode::Escape => map_to_escape,
        Mode::Suppress => map_suppress,
    };
    let mapping = AsciiMapping::new(&map_fn, exclude);

    let mut truncate = args.truncate;

    // Use stdin if no files are specified
    if args.files.is_empty() {
        // Early return if no more chars should be printed
        if truncate == 0 {
            return Ok(());
        }

        try_process(&mut io::stdin(), &mapping, &mut truncate)?;
    }

    for filename in &args.files {
        // Early return if no more chars should be printed, to avoid opening further files
        if truncate == 0 {
            return Ok(());
        }

        if filename == "-" {
            try_process(&mut io::stdin(), &mapping, &mut truncate)?;
            continue;
        }

        let file = File::open(filename);
        match file {
            Ok(file) => try_process(&mut BufReader::new(file), &mapping, &mut truncate)?,
            Err(err) => {
                eprintln!(
                    "{}: {}: {}",
                    env::args().next().expect("Cannot obtain executable name"),
                    filename,
                    err
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

/// Wrapper for process function, to handle SIGPIPE.
fn try_process<R: Read>(
    reader: &mut R,
    mapping: &AsciiMapping,
    truncate: &mut i128,
) -> Result<(), io::Error> {
    if let Err(e) = process(reader, mapping, truncate) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(141);
        }
        Err(e)?;
    }
    Ok(())
}

/// Read from input reader, perform conversion, and write to stdout.
fn process<R: Read>(
    reader: &mut R,
    mapping: &AsciiMapping,
    truncate: &mut i128,
) -> Result<(), io::Error> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    let mut buf: [u8; 16 * 1024] = [0; 16 * 1024];

    loop {
        let n = reader.read(&mut buf[..])?;
        if n == 0 {
            break; // no more input
        }

        if *truncate < 0 {
            // No truncate limit
            handle.write_all(mapping.convert_u8_slice(&buf, n).as_bytes())?;
        } else if *truncate > n as i128 {
            // Won't reach limit in this block
            handle.write_all(mapping.convert_u8_slice(&buf, n).as_bytes())?;
            *truncate -= n as i128;
        } else {
            // Will reach limit within this block
            #[allow(clippy::cast_sign_loss)]
            handle.write_all(
                mapping
                    .convert_u8_slice(&buf, *truncate as usize)
                    .as_bytes(),
            )?;
            *truncate = 0;
            break;
        }
        handle.flush()?;
    }
    Ok(())
}

/// Parses exclude string
fn parse_exclude(exclusions: Vec<String>) -> Result<[bool; 256], Box<dyn error::Error>> {
    // Initialize to false
    let mut exclude: [bool; 256] = [false; 256];

    // Don't print any non-printable characters
    if let [s] = exclusions.as_slice() && s.is_empty() {
        return Ok(exclude);
    }

    // Split by comma, parse into int, set index of exclude array
    for exclusion in exclusions {
        if let Ok(i) = str::parse::<u8>(&exclusion) {
            exclude[i as usize] = true;
        } else {
            Err(format!(
                "Error: Encountered unparsable value \"{exclusion}\" in exclusion list"
            ))?;
        }
    }
    Ok(exclude)
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}

#[cfg(test)]
mod cli {
    use std::io::Write;
    use std::process::{Command, Output, Stdio};

    fn get_program_path() -> String {
        // Adapted from cargo-script
        let target_dir =
            std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| String::from("target"));
        format!("{}/debug/safe-ascii", target_dir)
    }

    fn run_with_input(args: &[&str], input: &[u8]) -> Output {
        let program_path = get_program_path();
        let mut process = Command::new(&program_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        process.stdin.as_mut().unwrap().write_all(input).unwrap();
        process.wait_with_output().unwrap()
    }

    // Compares output of stdin and file inputs
    #[test]
    fn stdin_direct() {
        let program_path = get_program_path();

        // safe-ascii -t 1000 ./safe-ascii
        let file_output = Command::new(&program_path)
            .args([program_path.as_str(), "-t", "1000"])
            .output()
            .unwrap();

        // safe-ascii -t 1000 < ./safe-ascii
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

        assert_eq!(0, file_output.status.code().unwrap());
        assert_eq!(0, stdin_output.status.code().unwrap());
        assert_eq!(&file_output.stdout, &stdin_output.stdout);
    }

    #[test]
    fn files_many() {
        let program_path = get_program_path();

        let file_output = Command::new(&program_path)
            .args(["Cargo.toml"])
            .output()
            .unwrap();

        let file_output_double = Command::new(&program_path)
            .args(["Cargo.toml", "Cargo.toml", "Cargo.toml"])
            .output()
            .unwrap();

        assert_eq!(
            file_output.stdout.len() * 3,
            file_output_double.stdout.len()
        );
    }

    #[test]
    fn truncation() {
        let output = run_with_input(&["-t", "2"], &[0, 48, 48]);
        assert_eq!(0, output.status.code().unwrap());
        assert_eq!("(NUL)0", String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn mode_escape() {
        let output = run_with_input(&["-m", "escape"], &[0, 48]);
        assert_eq!(0, output.status.code().unwrap());
        assert_eq!("\\x00\\x30", String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn mode_mnemonic() {
        let output = run_with_input(&["-m", "mnemonic"], &[0, 48]);
        assert_eq!(0, output.status.code().unwrap());
        assert_eq!("(NUL)0", String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn mode_suppress() {
        let output = run_with_input(&["-m", "suppress"], &[0, 48]);
        assert_eq!(0, output.status.code().unwrap());
        assert_eq!("0", String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn bad_suppression_list() {
        let output = run_with_input(&["-x", "whatever"], &[]);

        assert_eq!(1, output.status.code().unwrap());
        assert_eq!("", String::from_utf8(output.stdout).unwrap());
        assert_eq!(
            "Error: Encountered unparsable value \"whatever\" in exclusion list\n",
            String::from_utf8(output.stderr).unwrap()
        );
    }

    #[test]
    fn non_existent_file() {
        let output = run_with_input(&["non-exist.file"], &[]);

        assert_eq!(1, output.status.code().unwrap());
        assert_eq!("", String::from_utf8(output.stdout).unwrap());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("non-exist.file: No such file or directory")
        );
    }

    #[test]
    fn empty_exclusion() {
        let output = run_with_input(&["-x", ""], &[0, 10, 32, 48]);
        assert_eq!(0, output.status.code().unwrap());
        assert_eq!("(NUL)(LF)(SP)0", String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn single_exclusion() {
        // Only newline (10) is excluded from conversion
        let output = run_with_input(&["-x", "10"], &[0, 10, 32, 48]);
        assert_eq!(0, output.status.code().unwrap());
        assert_eq!("(NUL)\n(SP)0", String::from_utf8(output.stdout).unwrap());
    }

    #[test]
    fn truncation_spans_buffer_boundary() {
        // Buffer size is 16KB (16384 bytes), truncate in the middle of second buffer
        let output = run_with_input(&["-t", "17000", "-m", "escape"], &[b'A'; 20000]);
        assert_eq!(0, output.status.code().unwrap());
        // Each byte becomes \xNN (4 chars)
        assert_eq!(output.stdout.len(), 17000 * 4);
    }

    #[test]
    fn empty_input() {
        let output = run_with_input(&[], &[]);
        assert_eq!(0, output.status.code().unwrap());
        assert!(output.stdout.is_empty());
    }
}
