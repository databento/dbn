use std::ops::Range;

use super::FromLittleEndianSlice;

/// Range of magic numbers for a Zstandard skippable frame.
pub(crate) const ZSTD_SKIPPABLE_MAGIC_RANGE: Range<u32> = 0x184D2A50..0x184D2A60;
/// Magic number for the beginning of a Zstandard frame.
const ZSTD_MAGIC_NUMBER: u32 = 0xFD2FB528;

pub fn starts_with_prefix(bytes: &[u8]) -> bool {
    if bytes.len() < 4 {
        return false;
    }
    let magic = u32::from_le_slice(&bytes[..4]);
    ZSTD_MAGIC_NUMBER == magic
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    use rstest::rstest;

    use super::*;
    use crate::{decode::tests::TEST_DATA_PATH, Schema};

    #[rstest]
    #[case::mbo(Schema::Mbo)]
    #[case::mbp1(Schema::Mbp1)]
    #[case::mbp10(Schema::Mbp10)]
    #[case::definition(Schema::Definition)]
    fn test_starts_with_prefix_valid(#[case] schema: Schema) {
        let mut file = File::open(format!("{TEST_DATA_PATH}/test_data.{schema}.dbn.zst")).unwrap();
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf).unwrap();
        assert!(starts_with_prefix(buf.as_slice()));
    }

    #[rstest]
    #[case::mbo(Schema::Mbo)]
    #[case::mbp1(Schema::Mbp1)]
    #[case::mbp10(Schema::Mbp10)]
    #[case::definition(Schema::Definition)]
    fn test_starts_with_prefix_other(#[case] schema: Schema) {
        let mut file = File::open(format!("{TEST_DATA_PATH}/test_data.{schema}.dbn")).unwrap();
        let mut buf = [0u8; 4];
        file.read_exact(&mut buf).unwrap();
        assert!(!starts_with_prefix(buf.as_slice()));
    }
}
