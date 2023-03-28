use std::{
    ffi::{c_char, CStr},
    io, slice,
};

use dbn::{
    encode::dbn::MetadataEncoder,
    enums::{rtype, SType, Schema},
    MetadataBuilder,
};

/// The byte offset of the `start` field in DBN-encoded Metadata.
pub const METADATA_START_OFFSET: usize = 26;
/// The minimum buffer size in bytes for encoding DBN Metadata.
pub const METADATA_MIN_ENCODED_SIZE: usize = 128;

/// Encodes DBN metadata to the given buffer.
///
/// # Errors
/// - Returns -1 if `buffer` is null.
/// - Returns -2 if `dataset` cannot be parsed.
/// - Returns -3 if the metadata cannot be encoded.
///
/// # Safety
/// This function assumes `dataset` is a valid pointer and `buffer` is of size
/// `length`.
#[no_mangle]
pub unsafe extern "C" fn encode_metadata(
    buffer: *mut c_char,
    length: libc::size_t,
    dataset: *const c_char,
    schema: Schema,
    start: u64,
) -> libc::c_int {
    if buffer.is_null() {
        return -1;
    }
    let dataset = match CStr::from_ptr(dataset).to_str() {
        Ok(dataset) => dataset.to_owned(),
        Err(_) => {
            return -2;
        }
    };
    let metadata = MetadataBuilder::new()
        .dataset(dataset)
        .start(start)
        .stype_in(SType::ProductId)
        .stype_out(SType::ProductId)
        .schema(schema)
        .build();
    let buffer: &mut [u8] = slice::from_raw_parts_mut(buffer as *mut u8, length);
    let mut cursor = io::Cursor::new(buffer);
    match MetadataEncoder::new(&mut cursor).encode(&metadata) {
        Ok(()) => cursor.position() as i32,
        Err(_) => -3,
    }
}

/// Tries to convert `rtype` to a [`Schema`](dbn::enums::Schema).
/// Returns `true` if `res` was set.
///
/// # Safety
/// Checks that `res` is not null before dereferencing it.
#[no_mangle]
pub unsafe extern "C" fn schema_from_rtype(rtype: u8, res: *mut Schema) -> bool {
    if res.is_null() {
        return false;
    }
    if let Some(schema) = rtype::try_into_schema(rtype) {
        *res = schema;
        true
    } else {
        false
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
