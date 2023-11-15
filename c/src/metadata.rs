use std::{
    ffi::{c_char, CStr},
    io, slice,
};

use dbn::{
    encode::dbn::MetadataEncoder,
    enums::{SType, Schema},
    MetadataBuilder,
};

/// The byte offset of the `start` field in DBN-encoded Metadata.
pub const METADATA_START_OFFSET: usize = 26;
/// The minimum buffer size in bytes for encoding DBN Metadata.
pub const METADATA_MIN_ENCODED_SIZE: usize = 128;

/// Encodes DBN metadata to the given buffer. Returns the number of bytes written.
///
/// # Errors
/// - Returns -1 if `buffer` is null.
/// - Returns -2 if `dataset` cannot be parsed.
/// - Returns -3 if the metadata cannot be encoded.
/// - Returns -4 if the version is invalid.
///
/// # Safety
/// This function assumes `dataset` is a valid pointer and `buffer` is of size
/// `length`.
#[no_mangle]
pub unsafe extern "C" fn encode_metadata(
    buffer: *mut c_char,
    length: libc::size_t,
    version: u8,
    dataset: *const c_char,
    schema: Schema,
    start: u64,
) -> libc::c_int {
    let buffer = if let Some(buffer) = (buffer as *mut u8).as_mut() {
        slice::from_raw_parts_mut(buffer, length)
    } else {
        return -1;
    };
    let dataset = match CStr::from_ptr(dataset).to_str() {
        Ok(dataset) => dataset.to_owned(),
        Err(_) => {
            return -2;
        }
    };
    if version == 0 || version > dbn::DBN_VERSION {
        return -4;
    }
    let metadata = MetadataBuilder::new()
        .version(version)
        .dataset(dataset)
        .start(start)
        .stype_in(Some(SType::InstrumentId))
        .stype_out(SType::InstrumentId)
        .schema(Some(schema))
        .build();
    let mut cursor = io::Cursor::new(buffer);
    match MetadataEncoder::new(&mut cursor).encode(&metadata) {
        Ok(()) => cursor.position() as i32,
        Err(_) => -3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // cbindgen doesn't support constants defined with expressions, so we test the equality here
    #[test]
    fn const_checks() {
        assert_eq!(
            METADATA_START_OFFSET,
            MetadataEncoder::<Vec<u8>>::START_OFFSET
        );
        assert_eq!(
            METADATA_MIN_ENCODED_SIZE,
            MetadataEncoder::<Vec<u8>>::MIN_ENCODED_SIZE
        );
    }
}
