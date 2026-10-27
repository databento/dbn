use crate::{SType, Schema, METADATA_FIXED_LEN, METADATA_PRELUDE_LEN};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MetadataLengths {
    pub dataset_cstr_len: usize,
    pub reserved_len: usize,
    pub has_record_count: bool,
    pub has_symbol_cstr_len: bool,
    pub has_layout: bool,
    pub should_pad_to_8: bool,
}

impl MetadataLengths {
    const V1: Self = Self {
        dataset_cstr_len: 16,
        reserved_len: 47,
        has_record_count: true,
        has_symbol_cstr_len: false,
        has_layout: false,
        should_pad_to_8: false,
    };
    const V2: Self = Self {
        reserved_len: 53,
        has_record_count: false,
        has_symbol_cstr_len: true,
        ..Self::V1
    };
    const V3: Self = Self {
        should_pad_to_8: true,
        ..Self::V2
    };
    const V4: Self = Self {
        dataset_cstr_len: 38,
        reserved_len: 31,
        has_layout: true,
        ..Self::V3
    };

    pub const fn for_version(version: u8) -> Option<&'static Self> {
        match version {
            0 | 1 => Some(&Self::V1),
            2 => Some(&Self::V2),
            3 => Some(&Self::V3),
            4 => Some(&Self::V4),
            _ => None,
        }
    }

    pub const fn start_offset(&self) -> usize {
        METADATA_PRELUDE_LEN + self.dataset_cstr_len + size_of::<Schema>()
    }

    const fn fixed_len(&self) -> usize {
        let u64_len = size_of::<u64>();
        let (start, end, limit, record_count) = (u64_len, u64_len, u64_len, u64_len);
        let (stype_in, stype_out, ts_out) =
            (size_of::<SType>(), size_of::<SType>(), size_of::<bool>());
        self.dataset_cstr_len
            + size_of::<Schema>()
            + start
            + end
            + limit
            + if self.has_record_count {
                record_count
            } else {
                0
            }
            + stype_in
            + stype_out
            + ts_out
            + if self.has_symbol_cstr_len {
                size_of::<u16>()
            } else {
                0
            }
            + self.reserved_len
    }
}

const _: () = {
    assert!(MetadataLengths::V1.fixed_len() == METADATA_FIXED_LEN);
    assert!(MetadataLengths::V2.fixed_len() == METADATA_FIXED_LEN);
    assert!(MetadataLengths::V3.fixed_len() == METADATA_FIXED_LEN);
    assert!(MetadataLengths::V4.fixed_len() == METADATA_FIXED_LEN);
    assert!(MetadataLengths::V4.start_offset().is_multiple_of(8));
};
