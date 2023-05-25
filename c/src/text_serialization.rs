use std::{
    ffi::c_char,
    io::{self, Write},
    slice,
};

use crate::cfile::CFileRef;
use dbn::{
    encode::{csv, json, DbnEncodable, EncodeDbn},
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
pub unsafe extern "C" fn s_serialize_record_header(
    buffer: *mut c_char,
    length: libc::size_t,
    record: *const RecordHeader,
    options: *const SerializeRecordOptions,
) -> libc::c_int {
    if buffer.is_null() {
        return SerializeError::NullBuffer as libc::c_int;
    }
    let buffer: &mut [u8] = slice::from_raw_parts_mut(buffer as *mut u8, length);
    if record.is_null() {
        return SerializeError::NullRecord as libc::c_int;
    }
    let record = RecordRef::unchecked_from_header(record);
    if options.is_null() {
        return SerializeError::NullOptions as libc::c_int;
    }
    let options = options.read();
    let mut cursor = io::Cursor::new(buffer);
    let res = match options.encoding {
        TextEncoding::Json => return 0,
        TextEncoding::Csv => {
            let mut encoder = csv::Encoder::new(&mut cursor, options.pretty_px, options.pretty_ts);
            rtype_ts_out_dispatch!(record, options.ts_out, serialize_csv_header, &mut encoder)
        }
    }
    .map_err(|e| anyhow::format_err!(e))
    // null byte
    .and_then(|_| Ok(cursor.write_all(&[0])?));
    if res.is_ok() {
        cursor.position() as i32
    } else {
        SerializeError::Serialization as libc::c_int
    }
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
    let mut cfile = if let Some(cfile) = CFileRef::new(file) {
        cfile
    } else {
        return SerializeError::NullFile as libc::c_int;
    };
    if record.is_null() {
        return SerializeError::NullRecord as libc::c_int;
    }
    let record = RecordRef::unchecked_from_header(record);
    if options.is_null() {
        return SerializeError::NullOptions as libc::c_int;
    }
    let options = options.read();
    let res = match options.encoding {
        TextEncoding::Json => {
            return 0;
        }
        TextEncoding::Csv => {
            let mut encoder = csv::Encoder::new(&mut cfile, options.pretty_px, options.pretty_ts);
            rtype_ts_out_dispatch!(record, options.ts_out, serialize_csv_header, &mut encoder)
        }
    };
    if res.is_ok() {
        cfile.bytes_written() as i32
    } else {
        SerializeError::Serialization as libc::c_int
    }
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
    if record.is_null() {
        return SerializeError::NullRecord as libc::c_int;
    }
    let record = RecordRef::unchecked_from_header(record);
    if options.is_null() {
        return SerializeError::NullOptions as libc::c_int;
    }
    let options = options.read();
    let mut cursor = io::Cursor::new(buffer);
    let res = match options.encoding {
        TextEncoding::Json => {
            json::Encoder::new(&mut cursor, false, options.pretty_px, options.pretty_ts)
                .encode_record_ref(record, options.ts_out)
        }
        TextEncoding::Csv => csv::Encoder::new(&mut cursor, options.pretty_px, options.pretty_ts)
            .encode_record_ref(record, options.ts_out),
    }
    // null byte
    .and_then(|_| Ok(cursor.write_all(&[0])?));
    if res.is_ok() {
        // subtract for null byte
        cursor.position() as i32 - 1
    } else {
        SerializeError::Serialization as libc::c_int
    }
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
    let mut cfile = if let Some(cfile) = CFileRef::new(file) {
        cfile
    } else {
        return SerializeError::NullFile as libc::c_int;
    };
    if record.is_null() {
        return SerializeError::NullRecord as libc::c_int;
    }
    let record = RecordRef::unchecked_from_header(record);
    if options.is_null() {
        return SerializeError::NullOptions as libc::c_int;
    }
    let options = options.read();
    let res = match options.encoding {
        TextEncoding::Json => {
            json::Encoder::new(&mut cfile, false, options.pretty_px, options.pretty_ts)
                .encode_record_ref(record, options.ts_out)
        }
        TextEncoding::Csv => csv::Encoder::new(&mut cfile, options.pretty_px, options.pretty_ts)
            .encode_record_ref(record, options.ts_out),
    };
    if res.is_ok() {
        cfile.bytes_written() as i32
    } else {
        SerializeError::Serialization as libc::c_int
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

fn serialize_csv_header<W: io::Write, R: DbnEncodable>(
    _rec: &R,
    encoder: &mut csv::Encoder<W>,
) -> anyhow::Result<()> {
    encoder.encode_header::<R>()
}