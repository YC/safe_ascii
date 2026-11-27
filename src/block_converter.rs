use safe_ascii::AsciiMapping;

const PADDED_MAPPING_WIDTH: usize = 8;

#[derive(Clone, Copy, Default)]
struct PaddedMapping {
    /// Padded byte representation of the mapping for this byte.
    padded_bytes: [u8; PADDED_MAPPING_WIDTH],
    /// Length of the mapping for this byte, without padding.
    unpadded_len: usize,
}

pub(super) struct BlockConverter {
    /// Padded mappings for each byte value.
    mappings: [PaddedMapping; 256],
    /// Preallocated output buffer for converted blocks.
    output: Vec<u8>,
}

impl BlockConverter {
    pub(super) fn new(block_size: usize, mapping: &AsciiMapping) -> Self {
        let mut mappings = [PaddedMapping::default(); 256];
        for byte in u8::MIN..=u8::MAX {
            // Convert the byte using the provided mapping
            let mapped = mapping.convert_u8(byte).as_bytes();
            assert!(
                mapped.len() <= PADDED_MAPPING_WIDTH,
                "Mapping for byte {byte} exceeds padded width"
            );

            // Store to padded entry
            let entry = &mut mappings[usize::from(byte)];
            entry.padded_bytes[..mapped.len()].copy_from_slice(mapped);
            entry.unpadded_len = mapped.len();
        }

        Self {
            mappings,
            output: vec![0; block_size * PADDED_MAPPING_WIDTH],
        }
    }

    /// Converts input no larger than the block size passed to `new`.
    pub(super) fn convert(&mut self, input: &[u8]) -> &[u8] {
        let mut output_len = 0;
        for &byte in input {
            // Copies include padding; the next mapping overwrites it.
            let entry = &self.mappings[usize::from(byte)];
            self.output[output_len..output_len + PADDED_MAPPING_WIDTH]
                .copy_from_slice(&entry.padded_bytes);
            output_len += entry.unpadded_len;
        }
        &self.output[..output_len]
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockConverter, PADDED_MAPPING_WIDTH};
    use crate::BLOCK_SIZE;
    use safe_ascii::{AsciiMapping, map_suppress, map_to_escape, map_to_mnemonic};

    #[test]
    fn block_conversion_matches_mapping() {
        let input: Vec<u8> = (u8::MIN..=u8::MAX).collect();

        let custom = |byte| match byte {
            1 => String::new(),
            2 => "12345678".to_owned(),
            3 => "\u{20ac}".to_owned(),
            _ => "x".to_owned(),
        };
        let map_fns: [fn(u8) -> String; 4] = [map_to_mnemonic, map_to_escape, map_suppress, custom];

        for map_fn in map_fns {
            let mapping = AsciiMapping::new(&map_fn, [false; 256]);
            let mut converter = BlockConverter::new(input.len(), &mapping);

            for block in [input.as_slice(), &[2, 1, 1], &[]] {
                let expected = mapping.convert_u8_slice(block, block.len());
                assert_eq!(converter.convert(block), expected.as_bytes());
            }
        }
    }

    #[test]
    fn converts_configured_block() {
        let mapping = AsciiMapping::new(&|_| "12345678".to_owned(), [false; 256]);

        let input = vec![b'A'; BLOCK_SIZE + 1];
        let mut converter = BlockConverter::new(input.len(), &mapping);

        assert_eq!(converter.convert(&input), b"12345678".repeat(input.len()));
    }

    #[test]
    #[should_panic(expected = "exceeds padded width")]
    fn block_conversion_rejects_long_mappings() {
        let mapping = AsciiMapping::new(
            &|byte| match byte {
                1 => "\u{263a}".repeat(PADDED_MAPPING_WIDTH),
                2 => String::new(),
                _ => "x".to_owned(),
            },
            [false; 256],
        );
        BlockConverter::new(BLOCK_SIZE, &mapping);
    }
}
