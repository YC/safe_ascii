use clap::{App, Arg};
use std::fs::File;
use std::io;

fn main() {
    // Command line arguments using clap
    let matches = App::new("safe_ascii")
        .version("1.0")
        .author("Steven Tang <steven@steventang.net>")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_name("mnemonic|escape|suppress")
                .possible_values(&["mnemonic", "escape", "suppress"])
                .about("'mnemonic' or \\x 'escape' sequence or 'suppress'")
                .takes_value(true)
                .multiple(false)
                .default_value("mnemonic"),
        )
        .arg(
            Arg::new("truncate")
                .short('t')
                .long("truncate")
                .value_name("truncate length")
                .about("length (bytes) to truncate at, -1 means no truncation")
                .takes_value(true)
                .multiple(false)
                .default_value("-1"),
        )
        .arg(
            Arg::new("exclude")
                .short('x')
                .long("exclude")
                .value_name("exclude characters")
                .about(
                    "comma-delimited decimal values of characters to print
                    (9 is HT (tab), 10 is NL (newline),
                    13 is CR (carriage return), 32 is SP (space))",
                )
                .takes_value(true)
                .default_value("10,32"),
        )
        .arg(Arg::new("files").multiple(true))
        .get_matches();

    // Extract command line arguments
    let mode = matches.value_of("mode").unwrap();
    let truncate = matches.value_of("truncate").unwrap();
    let mut truncate = str::parse::<i128>(truncate).expect("Cannot parse truncate");
    let exclude = parse_exclude(matches.value_of("exclude").unwrap());

    // If files are given, then use files; otherwise, use stdin
    match matches.values_of("files") {
        Some(values) => {
            // files
            for file in values {
                let file = File::open(file).unwrap();
                process_file(file, mode, &mut truncate, exclude);

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
fn parse_exclude(s: &str) -> [bool; 256] {
    // Initialize to false
    let mut exclude: [bool; 256] = [false; 256];
    // Split by comma, parse into int, set index of exclude array
    for i in s.split(',') {
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
                            print!("{}", map_to_mnemonic(c as char));
                        }
                        "escape" => {
                            print!("{}", map_to_escape(c as char));
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

// Map to mnemonic form
fn map_to_mnemonic(c: char) -> &'static str {
    match c as u8 {
        0 => "(NUL)",
        1 => "(SOH)",
        2 => "(STX)",
        3 => "(ETX)",
        4 => "(EOT)",
        5 => "(ENQ)",
        6 => "(ACK)",
        7 => "(BEL)",
        8 => "(BS)",
        9 => "(HT)",
        10 => "(LF)",
        11 => "(VT)",
        12 => "(FF)",
        13 => "(CR)",
        14 => "(SO)",
        15 => "(SI)",
        16 => "(DLE)",
        17 => "(DC1)",
        18 => "(DC2)",
        19 => "(DC3)",
        20 => "(DC4)",
        21 => "(NAK)",
        22 => "(SYN)",
        23 => "(ETB)",
        24 => "(CAN)",
        25 => "(EM)",
        26 => "(SUB)",
        27 => "(ESC)",
        28 => "(FS)",
        29 => "(GS)",
        30 => "(RS)",
        31 => "(US)",
        32 => "(SP)",
        33..=126 => "", // Printable
        127 => "(DEL)",
        128..=255 => "(>7F)",
    }
}

#[test]
fn map_to_mnemonic_test() {
    assert_eq!(map_to_mnemonic('\n'), "(LF)");
    assert_eq!(map_to_mnemonic('\0'), "(NUL)");
}

// Map to escape sequence form
fn map_to_escape(c: char) -> String {
    return format!("\\x{:02x}", c as u8);
}

#[test]
fn map_to_escape_test() {
    assert_eq!(map_to_escape('\n'), "\\x0a");
    assert_eq!(map_to_escape('0'), "\\x30");
}
