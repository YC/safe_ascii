#![crate_name = "safe_ascii"]

/// Returns a char's mnemonic representation.
///
/// * ASCII characters in range 0x21 to 0x7e are not escaped.
///
/// # Examples
///
/// ```
/// use safe_ascii;
///
/// assert_eq!(safe_ascii::map_to_mnemonic('\n' as u8), "(LF)");
/// assert_eq!(safe_ascii::map_to_mnemonic('\0' as u8), "(NUL)");
/// assert_eq!(safe_ascii::map_to_mnemonic('a' as u8), "a");
/// assert_eq!(safe_ascii::map_to_mnemonic('~' as u8), "~");
/// ```
pub fn map_to_mnemonic(c: u8) -> String {
    return match c {
        0 => "(NUL)".to_owned(),
        1 => "(SOH)".to_owned(),
        2 => "(STX)".to_owned(),
        3 => "(ETX)".to_owned(),
        4 => "(EOT)".to_owned(),
        5 => "(ENQ)".to_owned(),
        6 => "(ACK)".to_owned(),
        7 => "(BEL)".to_owned(),
        8 => "(BS)".to_owned(),
        9 => "(HT)".to_owned(),
        10 => "(LF)".to_owned(),
        11 => "(VT)".to_owned(),
        12 => "(FF)".to_owned(),
        13 => "(CR)".to_owned(),
        14 => "(SO)".to_owned(),
        15 => "(SI)".to_owned(),
        16 => "(DLE)".to_owned(),
        17 => "(DC1)".to_owned(),
        18 => "(DC2)".to_owned(),
        19 => "(DC3)".to_owned(),
        20 => "(DC4)".to_owned(),
        21 => "(NAK)".to_owned(),
        22 => "(SYN)".to_owned(),
        23 => "(ETB)".to_owned(),
        24 => "(CAN)".to_owned(),
        25 => "(EM)".to_owned(),
        26 => "(SUB)".to_owned(),
        27 => "(ESC)".to_owned(),
        28 => "(FS)".to_owned(),
        29 => "(GS)".to_owned(),
        30 => "(RS)".to_owned(),
        31 => "(US)".to_owned(),
        32 => "(SP)".to_owned(),
        33..=126 => (c as char).to_string(), // Printable
        127 => "(DEL)".to_owned(),
        128..=255 => "(>7F)".to_owned(),
    }
}

// Map to escape sequence form
pub fn map_to_escape(c: u8) -> String {
    return format!("\\x{:02x}", c);
}

#[test]
fn map_to_escape_test() {
    assert_eq!(map_to_escape('\0' as u8), "\\x00");
    assert_eq!(map_to_escape('\n' as u8), "\\x0a");
    assert_eq!(map_to_escape('0' as u8), "\\x30");
}
