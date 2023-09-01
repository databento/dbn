// RawFd isn't defined for windows
#![cfg(not(target_os = "windows"))]

use std::{
    fs::File,
    io::BufReader,
    os::fd::{FromRawFd, RawFd},
    ptr::{null, null_mut},
};

use dbn::{
    decode::{DecodeDbn, DecodeRecordRef, DynDecoder},
    enums::Compression,
    record::RecordHeader,
    Metadata,
};

pub type Decoder = DynDecoder<'static, BufReader<File>>;

/// Creates a DBN decoder. Returns null in case of error.
///
/// # Safety
/// `file` must be a valid file descriptor. This function assumes ownership of `file`.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_create(file: RawFd, compression: Compression) -> *mut Decoder {
    let decoder = match DynDecoder::new(File::from_raw_fd(file), compression) {
        Ok(d) => d,
        Err(_) => {
            return null_mut();
        }
    };
    Box::into_raw(Box::new(decoder))
}

/// Returns a pointer to the decoded DBN metadata.
///
/// # Safety
/// Verifies `decoder` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_metadata(decoder: *mut Decoder) -> *const Metadata {
    if let Some(metadata) = decoder.as_mut().map(|d| d.metadata()) {
        metadata
    } else {
        null()
    }
}

/// Decodes and returns a pointer to the next record.
///
/// # Safety
/// Verifies `decoder` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_decode(decoder: *mut Decoder) -> *const RecordHeader {
    if let Some(Ok(Some(rec))) = decoder.as_mut().map(|d| d.decode_record_ref()) {
        return rec.header();
    } else {
        null()
    }
}

/// Frees memory associated with the DBN decoder.
///
/// # Safety
/// Verifies `decoder` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_free(decoder: *mut Decoder) {
    if let Some(decoder) = decoder.as_mut() {
        drop(Box::from_raw(decoder));
    }
}
