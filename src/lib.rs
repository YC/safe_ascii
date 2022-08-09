#![crate_name = "safe_ascii"]

/// Type for storing precomputed mapping between u8 to String.
/// (Subject to change)
pub struct AsciiMapping {
    mapping: [String; 256],
}

impl AsciiMapping {
    /// Generates a mapping table from u8 to string.
    ///
    /// ```
    /// use safe_ascii::AsciiMapping;
    /// let mut exclude: [bool; 256] = [false; 256];
    /// let _ = AsciiMapping::new(&safe_ascii::map_to_mnemonic, exclude);
    /// ```
    pub fn new(map_fn: &dyn Fn(u8) -> String, exclusion_list: [bool; 256]) -> Self {
        // https://stackoverflow.com/questions/28656387
        let mut result: [String; 256] = [(); 256].map(|_| String::default());

        for i in 0u8..=255 {
            if exclusion_list[i as usize] {
                result[i as usize] = (i as char).to_string();
            } else {
                result[i as usize] = map_fn(i);
            }
        }

        AsciiMapping { mapping: result }
    }

    /// Convert a u8 according to the mapping.
    ///
    /// ```
    /// use safe_ascii::AsciiMapping;
    /// let mut exclude: [bool; 256] = [false; 256];
    /// let mapping = AsciiMapping::new(&safe_ascii::map_to_mnemonic, exclude);
    /// assert_eq!(mapping.convert_u8(0), "(NUL)");
    /// ```
    pub fn convert_u8(&self, input: u8) -> &str {
        &self.mapping[input as usize]
    }

    /// Convert a u8 according to the mapping.
    ///
    /// ```
    /// use safe_ascii::AsciiMapping;
    /// let mut exclude: [bool; 256] = [false; 256];
    /// let mapping = AsciiMapping::new(&safe_ascii::map_to_mnemonic, exclude);
    /// assert_eq!(mapping.convert_u8_slice(&['h' as u8, ' ' as u8, 'i' as u8]), "h(SP)i");
    /// ```
    pub fn convert_u8_slice(&self, input: &[u8]) -> String {
        input
            .iter()
            .map(|c| self.mapping[*c as usize].as_ref())
            .collect::<Vec<&str>>()
            .join("")
    }
}

#[test]
fn test_generate_mapping() {
    // Exclusion list with all but first excluded
    let mut exclusion_list: [bool; 256] = [true; 256];
    exclusion_list[1] = false;

    let mapping = AsciiMapping::new(&map_to_mnemonic, exclusion_list);
    assert_eq!(mapping.mapping[0], "\0");
    assert_eq!(mapping.mapping[1], "(SOH)");
    assert_eq!(mapping.mapping[48], "0");
}

/// Returns a char's mnemonic representation.
///
/// * ASCII characters in range 0x21 to 0x7e are not escaped.
///
/// # Examples
///
/// ```
/// use safe_ascii;
///
/// assert_eq!(safe_ascii::map_to_mnemonic('\0' as u8), "(NUL)");
/// assert_eq!(safe_ascii::map_to_mnemonic('\n' as u8), "(LF)");
/// assert_eq!(safe_ascii::map_to_mnemonic('\r' as u8), "(CR)");
/// assert_eq!(safe_ascii::map_to_mnemonic('a' as u8), "a");
/// assert_eq!(safe_ascii::map_to_mnemonic('~' as u8), "~");
/// ```
pub fn map_to_mnemonic(c: u8) -> String {
    match c {
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

/// Returns a char's escape sequence representation.
///
/// * ASCII characters in range 0x20 to 0x7e are not escaped.
///
/// # Examples
///
/// ```
/// use safe_ascii;
///
/// assert_eq!(safe_ascii::map_to_escape('\0' as u8), "\\0");
/// assert_eq!(safe_ascii::map_to_escape('\t' as u8), "\\t");
/// assert_eq!(safe_ascii::map_to_escape('\n' as u8), "\\n");
/// assert_eq!(safe_ascii::map_to_escape('\r' as u8), "\\r");
/// assert_eq!(safe_ascii::map_to_escape('0' as u8), "0");
/// assert_eq!(safe_ascii::map_to_escape('a' as u8), "a");
/// assert_eq!(safe_ascii::map_to_escape('~' as u8), "~");
/// ```
// Map to escape sequence form
pub fn map_to_escape(c: u8) -> String {
    // Note: some escape codes are not covered
    // https://github.com/rust-lang/rfcs/pull/1437

    match c {
        0 => "\\0".to_owned(),
        9 => "\\t".to_owned(),
        10 => "\\n".to_owned(),
        13 => "\\r".to_owned(),
        32..=126 => (c as char).to_string(), // Printable
        _ => format!("\\x{:02x}", c),
    }
}
