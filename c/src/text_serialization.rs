use std::{
    ffi::c_char,
    io::{self, Write},
    mem, slice,
};

use crate::cfile::CFileRef;
use dbn::{
    compat::InstrumentDefMsgV2,
    encode::{csv, json, DbnEncodable, EncodeRecord, EncodeRecordRef},
    enums::{rtype, Schema},
    record::RecordHeader,
    record_ref::RecordRef,
    rtype_ts_out_dispatch,
};

/// The encoding to serialize as.
#[repr(C)]
pub enum TextEncoding {
    Csv,
    Json,
}

#[repr(C)]
pub enum SerializeError {
    NullBuffer = -1,
    NullFile = -2,
    NullRecord = -3,
    NullOptions = -4,
    Serialization = -5,
}

/// Options for serializing records.
#[repr(C)]
pub struct SerializeRecordOptions {
    /// The output encoding.
    encoding: TextEncoding,
    /// Whether to include the `ts_out` field with the serialization time.
    ts_out: bool,
    /// If `true`, prices are converted to decimal strings.
    pretty_px: bool,
    /// If `true`, timestamps are converted to ISO datetime strings.
    pretty_ts: bool,
}

/// Writes the header to `buffer` if the specified encoding is CSV, otherwise is a no-op.
/// Returns the number of bytes written.
///
/// # Errors
/// - Returns -1 if `buffer` is null.
/// - Returns -3 if `record` is null.
/// - Returns -4 if `options` is null.
/// - Returns -5 if there's an error serializing.
///
/// # Safety
/// This function assumes `buffer` is of size `length`. It checks the validity of all
/// pointers before dereferencing.
#[no_mangle]
pub unsafe extern "C" fn s_serialize_record_header(
    buffer: *mut c_char,
    length: libc::size_t,
    record: *const RecordHeader,
    options: *const SerializeRecordOptions,
) -> libc::c_int {
    let buffer = if let Some(buffer) = (buffer as *mut u8).as_mut() {
        slice::from_raw_parts_mut(buffer, length)
    } else {
        return SerializeError::NullBuffer as libc::c_int;
    };
    let record = if let Some(record) = record.as_ref() {
        RecordRef::unchecked_from_header(record)
    } else {
        return SerializeError::NullRecord as libc::c_int;
    };
    let Some(options) = options.as_ref() else {
        return SerializeError::NullOptions as libc::c_int;
    };
    let mut cursor = io::Cursor::new(buffer);
    let res = match options.encoding {
        TextEncoding::Json => return 0,
        TextEncoding::Csv => {
            let mut encoder = csv::Encoder::new(&mut cursor, options.pretty_px, options.pretty_ts);
            rtype_ts_out_dispatch!(record, options.ts_out, serialize_csv_header, &mut encoder)
        }
    }
    // flatten
    .and_then(|res| res);
    write_null_and_ret(cursor, res)
}

/// Serializes the header to the C file stream if the specified encoding is CSV,
/// otherwise is a no-op. Returns the number of bytes written.
///
/// # Errors
/// - Returns -2 if `file` is null.
/// - Returns -3 if `record` is null.
/// - Returns -4 if `options` is null.
/// - Returns -5 if there's an error serializing.
///
/// # Safety
/// Checks the validity of all pointers before dereferencing.
#[no_mangle]
pub unsafe extern "C" fn f_serialize_record_header(
    file: *mut libc::FILE,
    record: *const RecordHeader,
    options: *const SerializeRecordOptions,
) -> libc::c_int {
    let Some(mut cfile) = CFileRef::new(file) else {
        return SerializeError::NullFile as libc::c_int;
    };
    let record = if let Some(record) = record.as_ref() {
        RecordRef::unchecked_from_header(record)
    } else {
        return SerializeError::NullRecord as libc::c_int;
    };
    let Some(options) = options.as_ref() else {
        return SerializeError::NullOptions as libc::c_int;
    };
    let res = match options.encoding {
        TextEncoding::Json => {
            return 0;
        }
        TextEncoding::Csv => {
            let mut encoder = csv::Encoder::new(&mut cfile, options.pretty_px, options.pretty_ts);
            rtype_ts_out_dispatch!(record, options.ts_out, serialize_csv_header, &mut encoder)
        }
    };
    res.map(|_| cfile.bytes_written() as i32)
        .unwrap_or(SerializeError::Serialization as libc::c_int)
}

/// Serializes `record` to the specified text encoding, writing the output to `buffer`.
/// Returns the number of bytes written.
///
/// # Errors
/// - Returns -1 if `buffer` is null.
/// - Returns -3 if `record` is null.
/// - Returns -4 if `options` is null.
/// - Returns -5 if there's an error serializing.
///
/// # Safety
/// This function assumes `buffer` is of size `length`. It checks the validity of all
/// pointers before dereferencing.
#[no_mangle]
pub unsafe extern "C" fn s_serialize_record(
    buffer: *mut c_char,
    length: libc::size_t,
    record: *const RecordHeader,
    options: *const SerializeRecordOptions,
) -> libc::c_int {
    if buffer.is_null() {
        return SerializeError::NullBuffer as libc::c_int;
    }
    let buffer: &mut [u8] = slice::from_raw_parts_mut(buffer as *mut u8, length);
    let record = if let Some(record) = record.as_ref() {
        RecordRef::unchecked_from_header(record)
    } else {
        return SerializeError::NullRecord as libc::c_int;
    };
    let Some(options) = options.as_ref() else {
        return SerializeError::NullOptions as libc::c_int;
    };
    let mut cursor = io::Cursor::new(buffer);
    // TODO(carter): reverse when V2 becomes the default
    if record.record_size() >= mem::size_of::<InstrumentDefMsgV2>() {
        if let Some(def_v2) = record.get::<InstrumentDefMsgV2>() {
            let res = match options.encoding {
                TextEncoding::Json => {
                    json::Encoder::new(&mut cursor, false, options.pretty_px, options.pretty_ts)
                        .encode_record(def_v2)
                }
                TextEncoding::Csv => {
                    csv::Encoder::new(&mut cursor, options.pretty_px, options.pretty_ts)
                        .encode_record(def_v2)
                }
            };
            return write_null_and_ret(cursor, res);
        }
    };
    let res = match options.encoding {
        TextEncoding::Json => {
            json::Encoder::new(&mut cursor, false, options.pretty_px, options.pretty_ts)
                .encode_record_ref_ts_out(record, options.ts_out)
        }
        TextEncoding::Csv => csv::Encoder::new(&mut cursor, options.pretty_px, options.pretty_ts)
            .encode_record_ref_ts_out(record, options.ts_out),
    };
    write_null_and_ret(cursor, res)
}

/// Serializes `record` to the C file stream. Returns the number of bytes written.
///
/// # Errors
/// - Returns -2 if `file` is null.
/// - Returns -3 if `record` is null.
/// - Returns -4 if `options` is null.
/// - Returns -5 if there's an error serializing.
///
/// # Safety
/// Checks the validity of all pointers before dereferencing.
#[no_mangle]
pub unsafe extern "C" fn f_serialize_record(
    file: *mut libc::FILE,
    record: *const RecordHeader,
    options: *const SerializeRecordOptions,
) -> libc::c_int {
    let Some(mut cfile) = CFileRef::new(file) else {
        return SerializeError::NullFile as libc::c_int;
    };
    let record = if let Some(record) = record.as_ref() {
        RecordRef::unchecked_from_header(record)
    } else {
        return SerializeError::NullRecord as libc::c_int;
    };
    let Some(options) = options.as_ref() else {
        return SerializeError::NullOptions as libc::c_int;
    };
    let res = match options.encoding {
        TextEncoding::Json => {
            json::Encoder::new(&mut cfile, false, options.pretty_px, options.pretty_ts)
                .encode_record_ref_ts_out(record, options.ts_out)
        }
        TextEncoding::Csv => csv::Encoder::new(&mut cfile, options.pretty_px, options.pretty_ts)
            .encode_record_ref_ts_out(record, options.ts_out),
    };
    res.map(|_| cfile.bytes_written() as i32)
        .unwrap_or(SerializeError::Serialization as libc::c_int)
}

/// Tries to convert `rtype` to a [`Schema`].
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

fn serialize_csv_header<W: io::Write, R: DbnEncodable>(
    _rec: &R,
    encoder: &mut csv::Encoder<W>,
) -> dbn::Result<()> {
    encoder.encode_header::<R>(false)
}

fn write_null_and_ret(mut cursor: io::Cursor<&mut [u8]>, res: dbn::Result<()>) -> libc::c_int {
    let res = res.and_then(|_| {
        cursor
            .write_all(&[0])
            .map_err(|e| dbn::Error::io(e, "writing null byte"))
    });
    // subtract 1 for null byte
    res.map(|_| cursor.position() as i32 - 1)
        .unwrap_or(SerializeError::Serialization as libc::c_int)
}

#[cfg(test)]
mod tests {
    use std::os::raw::c_char;

    use dbn::InstrumentDefMsg;

    use super::*;

    #[test]
    fn test_serialize_def_v1() {
        // TODO(carter): update once DBNv2 is the default
        let mut def_v1 = InstrumentDefMsg::default();
        def_v1.raw_symbol = [b'a' as c_char; dbn::compat::SYMBOL_CSTR_LEN_V1];
        def_v1.raw_symbol[dbn::compat::SYMBOL_CSTR_LEN_V1 - 1] = 0;
        let mut buf = [0; 5000];
        assert!(
            unsafe {
                s_serialize_record(
                    buf.as_mut_ptr().cast(),
                    buf.len(),
                    &def_v1.hd,
                    &SerializeRecordOptions {
                        encoding: TextEncoding::Json,
                        ts_out: false,
                        pretty_px: false,
                        pretty_ts: false,
                    },
                )
            } > 0
        );
        let res = std::str::from_utf8(buf.as_slice()).unwrap();
        assert!(res.contains(&format!(
            "\"raw_symbol\":\"{}\",",
            "a".repeat(dbn::compat::SYMBOL_CSTR_LEN_V1 - 1)
        )));
    }

    #[test]
    fn test_serialize_def_v2() {
        let mut def_v2 = InstrumentDefMsgV2::from(&InstrumentDefMsg::default());
        def_v2.raw_symbol = [b'a' as c_char; dbn::compat::SYMBOL_CSTR_LEN_V2];
        def_v2.raw_symbol[dbn::compat::SYMBOL_CSTR_LEN_V2 - 1] = 0;
        let mut buf = [0; 5000];
        assert!(
            unsafe {
                s_serialize_record(
                    buf.as_mut_ptr().cast(),
                    buf.len(),
                    &def_v2.hd,
                    &SerializeRecordOptions {
                        encoding: TextEncoding::Json,
                        ts_out: false,
                        pretty_px: false,
                        pretty_ts: false,
                    },
                )
            } > 0
        );
        let res = std::str::from_utf8(buf.as_slice()).unwrap();
        assert!(res.contains(&format!(
            "\"raw_symbol\":\"{}\",",
            "a".repeat(dbn::compat::SYMBOL_CSTR_LEN_V2 - 1)
        )));
    }
}
