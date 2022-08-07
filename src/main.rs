use safe_ascii::{map_to_escape, map_to_mnemonic};
use clap::{Arg, Values, Command};
use std::{env, fs::File, io::{self, BufReader}};

fn main() {
    // Command line arguments using clap
    let matches = Command::new("safe-ascii")
        .version("1.0.5")
        .about("A tool for sanitising ASCII files to printable characters.")
        .author("Steven Tang <yc@steventang.net>")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("mnemonic|escape|suppress")
                .possible_values(&["mnemonic", "escape", "suppress"])
                .long_help(
                    "mnemonic: abbreviation e.g. (NUL), (SP), (NL)
escape: \\x sequence, e.g. \\x00 \\x20, \\x0a
suppress: don't print non-printable characters",
                )
                .takes_value(true)
                .multiple_values(false)
                .default_value("mnemonic"),
        )
        .arg(
            Arg::new("truncate")
                .short('t')
                .long("truncate")
                .value_name("truncate length")
                .long_help("length (bytes) to truncate at, -1 means no truncation")
                .takes_value(true)
                .multiple_values(false)
                .default_value("-1"),
        )
        .arg(
            Arg::new("exclude")
                .short('x')
                .long("exclude")
                .value_name("exclude characters")
                .long_help(
                    "comma-delimited decimal values of characters to print
(9 is HT (tab), 10 is NL (newline), 13 is CR (carriage return), 32 is SP (space))",
                )
                .multiple_values(false)
                .required(false)
                .value_delimiter(',')
                .default_value("10,32"),
        )
        .arg(Arg::new("files").multiple_values(true))
        .get_matches();

    // Extract command line arguments
    let mode = matches.value_of("mode").expect("Cannot read mode");
    let truncate = matches.value_of("truncate").expect("Cannot read truncate");
    let mut truncate = str::parse::<i128>(truncate).expect("Cannot parse truncate");
    let exclude = parse_exclude(
        matches
            .values_of("exclude")
            .map(Values::collect)
            .unwrap_or_default(),
    );

    // If files are given, then use files; otherwise, use stdin
    match matches.values_of("files") {
        Some(values) => {
            // files
            for filename in values {
                // stdin
                if filename == "-" {
                    process_file(io::stdin(), mode, &mut truncate, exclude);
                    continue;
                }

                let file = File::open(filename);
                match file {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        process_file(reader, mode, &mut truncate, exclude);
                    }
                    Err(err) => {
                        eprintln!(
                            "{}: {}: {}",
                            env::args().next().expect("Cannot obtain executable name"),
                            filename,
                            err
                        )
                    }
                }

                // Early return if no more chars should be printed
                if truncate == 0 {
                    break;
                }
            }
        }
        None => {
            // stdin
            process_file(io::stdin(), mode, &mut truncate, exclude);
        }
    }
}

// Parses exclude string
fn parse_exclude(s: Vec<&str>) -> [bool; 256] {
    // Initialize to false
    let mut exclude: [bool; 256] = [false; 256];
    // Split by comma, parse into int, set index of exclude array
    for i in s {
        if let Ok(i) = str::parse::<u8>(i) {
            exclude[i as usize] = true;
        }
    }
    exclude
}

// Process files with Read trait
fn process_file<R: io::Read>(f: R, mode: &str, truncate: &mut i128, exclude: [bool; 256]) {
    for b in f.bytes() {
        match b {
            Ok(c) => {
                // Printable or excluded
                if (33..=126).contains(&c) || exclude[c as usize] {
                    print!("{}", c as char)
                } else {
                    match mode {
                        "mnemonic" => {
                            print!("{}", map_to_mnemonic(c));
                        }
                        "escape" => {
                            print!("{}", map_to_escape(c));
                        }
                        _ => (),
                    }
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
}

#[cfg(test)]
mod cli {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Compares output of stdin and file inputs
    #[test]
    fn stdin_file() {
        // ./safe-ascii -t 1000 ./safe-ascii
        let file_output = Command::new("./target/debug/safe-ascii")
            .args(["./target/debug/safe-ascii", "-t", "1000"])
            .output()
            .unwrap();

        // ./safe-ascii -t 1000 < ./safe-ascii
        let file = std::fs::read("./target/debug/safe-ascii").unwrap();
        // https://stackoverflow.com/a/49597789
        let mut stdin_process = Command::new("./target/debug/safe-ascii")
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
