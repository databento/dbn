//! A push-based DBN decoder built directly on [`DbnFsm`]. The caller owns the read
//! loop and any decompression, feeding raw (decompressed) bytes in and draining
//! decoded metadata and records out.

use std::{
    ffi::{c_char, CString},
    ptr::{null, null_mut},
    slice,
};

use dbn::{
    decode::dbn::fsm::{DbnFsm, ProcessResult},
    Metadata, RecordHeader, VersionUpgradePolicy,
};

/// A push-based DBN decoder. Create with `DbnDecoder_create`, feed bytes with
/// `DbnDecoder_space` and `DbnDecoder_fill` (or `DbnDecoder_write_all`), and drain with
/// `DbnDecoder_process`.
pub struct Decoder {
    fsm: DbnFsm,
    last_error: Option<CString>,
}

/// Options for creating a [`Decoder`].
#[repr(C)]
pub struct DecoderOptions {
    /// The `VersionUpgradePolicy` discriminant.
    pub upgrade_policy: u8,
    /// Whether records are expected to have `ts_out` appended.
    pub ts_out: bool,
    /// The expected input DBN version, or 0 to detect it. Only applicable with
    /// `skip_metadata`, otherwise it's overwritten by the version in the metadata.
    pub input_version: u8,
    /// Whether the stream has no metadata header (skip parsing one).
    pub skip_metadata: bool,
    /// Buffer size in bytes, or 0 to use the default. With the
    /// `DbnDecoder_space`-`DbnDecoder_fill` path the buffer has to be big enough to
    /// hold the largest record in the stream. `DbnDecoder_write_all`
    /// grows the buffer as needed.
    pub buffer_size: usize,
}

/// The reason `DbnDecoder_create` failed, written to its `error` out-param.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum DecoderError {
    /// The `options` pointer was null.
    NullOptions,
    /// `upgrade_policy` was not a valid `VersionUpgradePolicy` discriminant.
    InvalidUpgradePolicy,
    /// `input_version` was nonzero and outside the supported range.
    InvalidInputVersion,
    /// The combination of `upgrade_policy` and `input_version` is unsupported.
    IncompatiblePolicyAndVersion,
}

/// The outcome of a call to `DbnDecoder_process`.
#[repr(C)]
pub enum ProcessStatus {
    /// More data should be read into `DbnDecoder_space` before processing again.
    ReadMore,
    /// Decoded the metadata header. Ownership of the `Metadata` out-pointer is
    /// transferred to the caller, who must free it with `DbnMetadata_free`.
    Metadata,
    /// Decoded a record, accessible via `DbnDecoder_last_record`.
    Record,
    /// Failed to decode. The message is available via `DbnDecoder_last_error`.
    Error,
}

/// Creates a push decoder from `options`. Returns null on error, writing the reason to
/// `error` when it is non-null.
///
/// # Safety
/// `options` must be a valid pointer to a `DbnDecoderOptions`. `error`, if not null,
/// must be a valid pointer. The returned pointer must be freed with `DbnDecoder_free`.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_create(
    options: *const DecoderOptions,
    error: *mut DecoderError,
) -> *mut Decoder {
    let Some(options) = options.as_ref() else {
        if let Some(error) = error.as_mut() {
            *error = DecoderError::NullOptions;
        }
        return null_mut();
    };
    let Ok(upgrade_policy) = VersionUpgradePolicy::try_from(options.upgrade_policy) else {
        if let Some(error) = error.as_mut() {
            *error = DecoderError::InvalidUpgradePolicy;
        }
        return null_mut();
    };
    let mut builder = DbnFsm::builder()
        .upgrade_policy(upgrade_policy)
        .ts_out(options.ts_out)
        .skip_metadata(options.skip_metadata);
    if options.buffer_size != 0 {
        builder = builder.buffer_size(options.buffer_size);
    }
    builder = match builder
        .input_dbn_version((options.input_version != 0).then_some(options.input_version))
    {
        Ok(builder) => builder,
        Err(_) => {
            if let Some(error) = error.as_mut() {
                *error = DecoderError::InvalidInputVersion;
            }
            return null_mut();
        }
    };
    match builder.build() {
        Ok(fsm) => Box::into_raw(Box::new(Decoder {
            fsm,
            last_error: None,
        })),
        Err(_) => {
            if let Some(error) = error.as_mut() {
                *error = DecoderError::IncompatiblePolicyAndVersion;
            }
            null_mut()
        }
    }
}

/// Returns a pointer to the decoder's writable buffer and writes its length to `len`.
/// Read up to `*len` bytes into it, then call `DbnDecoder_fill` with the number of
/// bytes actually written.
///
/// # Safety
/// `decoder` and `len` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_space(decoder: *mut Decoder, len: *mut usize) -> *mut u8 {
    let Some(decoder) = decoder.as_mut() else {
        return null_mut();
    };
    let space = decoder.fsm.space();
    if let Some(len) = len.as_mut() {
        *len = space.len();
    }
    space.as_mut_ptr()
}

/// Indicates that `nbytes` were written into the buffer returned by `DbnDecoder_space`.
///
/// # Safety
/// Verifies `decoder` is not null. `nbytes` must not exceed the length returned
/// by the preceding `DbnDecoder_space` call.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_fill(decoder: *mut Decoder, nbytes: usize) {
    if let Some(decoder) = decoder.as_mut() {
        decoder.fsm.fill(nbytes);
    }
}

/// Copies `length` bytes from `data` into the decoder's buffer. A copying alternative
/// to `DbnDecoder_space` and `DbnDecoder_fill`.
///
/// # Safety
/// Verifies `decoder` is not null. `data` must point to at least `length` bytes.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_write_all(
    decoder: *mut Decoder,
    data: *const u8,
    length: usize,
) {
    let Some(decoder) = decoder.as_mut() else {
        return;
    };
    if data.is_null() {
        return;
    }
    decoder.fsm.write_all(slice::from_raw_parts(data, length));
}

/// Processes buffered data, returning the outcome. Should be called repeatedly until
/// `ReadMore` is returned, at which point more data should be read in.
///
/// On `ReadMore`, `read_more` is set to the minimum additional bytes needed. On
/// `Metadata`, `metadata` is set to an owned `Metadata` the caller must free with
/// `DbnMetadata_free`, or the decoded metadata is dropped if `metadata` is null. On
/// `Record`, use `DbnDecoder_last_record`. On `Error`, use `DbnDecoder_last_error`.
///
/// # Safety
/// Verifies `decoder` is not null. `read_more` and `metadata`, if not null, must be
/// valid pointers.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_process(
    decoder: *mut Decoder,
    read_more: *mut usize,
    metadata: *mut *mut Metadata,
) -> ProcessStatus {
    let Some(decoder) = decoder.as_mut() else {
        return ProcessStatus::Error;
    };
    match decoder.fsm.process() {
        ProcessResult::ReadMore(nbytes) => {
            if let Some(read_more) = read_more.as_mut() {
                *read_more = nbytes;
            }
            ProcessStatus::ReadMore
        }
        ProcessResult::Metadata(m) => {
            if let Some(metadata) = metadata.as_mut() {
                *metadata = Box::into_raw(Box::new(m));
            }
            ProcessStatus::Metadata
        }
        ProcessResult::Record(()) => ProcessStatus::Record,
        ProcessResult::Err(err) => {
            decoder.last_error = CString::new(err.to_string()).ok();
            ProcessStatus::Error
        }
    }
}

/// Returns a pointer to the most recently decoded record's header, or null if there is
/// none. The pointer is valid until the next call to `DbnDecoder_process` or a buffer
/// mutation.
///
/// # Safety
/// Verifies `decoder` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_last_record(decoder: *mut Decoder) -> *const RecordHeader {
    decoder
        .as_ref()
        .and_then(|d| d.fsm.last_record())
        .map_or(null(), |rec| rec.header() as *const RecordHeader)
}

/// Returns a pointer to the decoder's unprocessed buffered bytes and writes the length
/// to `len`. The pointer is valid until the next buffer mutation.
///
/// # Safety
/// `decoder` and `len` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_data(decoder: *const Decoder, len: *mut usize) -> *const u8 {
    let Some(decoder) = decoder.as_ref() else {
        return null();
    };
    let data = decoder.fsm.data();
    if let Some(len) = len.as_mut() {
        *len = data.len();
    }
    data.as_ptr()
}

/// Returns the message from the most recent `Error` result as a null-terminated string,
/// or null if there has never been one. The message remains until overwritten by a
/// later error.
///
/// # Safety
/// Verifies `decoder` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_last_error(decoder: *const Decoder) -> *const c_char {
    decoder
        .as_ref()
        .and_then(|d| d.last_error.as_ref())
        .map_or(null(), |err| err.as_ptr())
}

/// Resets the decoder to expect DBN metadata so the same decoder can be used for
/// another stream. Any buffered data and the last error are discarded.
///
/// # Safety
/// Verifies `decoder` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_reset(decoder: *mut Decoder) {
    if let Some(decoder) = decoder.as_mut() {
        decoder.fsm.reset();
        decoder.last_error = None;
    }
}

/// Frees memory associated with the push decoder.
///
/// # Safety
/// Verifies `decoder` is not null. `decoder` must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn DbnDecoder_free(decoder: *mut Decoder) {
    if !decoder.is_null() {
        drop(Box::from_raw(decoder));
    }
}

#[cfg(test)]
mod tests {
    use dbn::{
        encode::{DbnEncoder, EncodeRecord},
        rtype, Metadata, RecordHeader, SType, Schema, TradeMsg,
    };
    use rstest::rstest;

    use super::*;

    fn sample_stream() -> (Vec<u8>, Vec<u32>) {
        let metadata = Metadata::builder()
            .dataset("GLBX.MDP3")
            .schema(Some(Schema::Trades))
            .start(0)
            .stype_in(Some(SType::InstrumentId))
            .stype_out(SType::InstrumentId)
            .build();
        let instrument_ids = vec![100, 101, 102];
        let mut buffer = Vec::new();
        let mut encoder = DbnEncoder::new(&mut buffer, &metadata).unwrap();
        for &instrument_id in &instrument_ids {
            let trade = TradeMsg {
                hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, instrument_id, 0),
                ..Default::default()
            };
            encoder.encode_record(&trade).unwrap();
        }
        (buffer, instrument_ids)
    }

    fn options(upgrade_policy: u8, input_version: u8) -> DecoderOptions {
        DecoderOptions {
            upgrade_policy,
            ts_out: false,
            input_version,
            skip_metadata: false,
            buffer_size: 0,
        }
    }

    #[rstest]
    fn round_trip_in_chunks(#[values(false, true)] use_space_fill: bool) {
        let (buffer, expected_ids) = sample_stream();
        let options = options(VersionUpgradePolicy::AsIs as u8, 0);
        let decoder = unsafe { DbnDecoder_create(&options, null_mut()) };
        assert!(!decoder.is_null());

        let mut got_dataset = None;
        let mut decoded_ids = Vec::new();
        let mut remaining = buffer.as_slice();
        unsafe {
            loop {
                loop {
                    let mut read_more = 0;
                    let mut metadata: *mut Metadata = null_mut();
                    match DbnDecoder_process(decoder, &mut read_more, &mut metadata) {
                        ProcessStatus::ReadMore => break,
                        ProcessStatus::Metadata => {
                            got_dataset = Some((*metadata).dataset.clone());
                            drop(Box::from_raw(metadata));
                        }
                        ProcessStatus::Record => {
                            let header = DbnDecoder_last_record(decoder);
                            assert!(!header.is_null());
                            decoded_ids.push((*header).instrument_id);
                        }
                        ProcessStatus::Error => {
                            panic!("decode error: unexpected");
                        }
                    }
                }
                if remaining.is_empty() {
                    break;
                }
                let n = remaining.len().min(13);
                if use_space_fill {
                    let mut space_len = 0;
                    let space = DbnDecoder_space(decoder, &mut space_len);
                    assert!(!space.is_null());
                    assert!(space_len >= n);
                    space.copy_from_nonoverlapping(remaining.as_ptr(), n);
                    DbnDecoder_fill(decoder, n);
                } else {
                    DbnDecoder_write_all(decoder, remaining.as_ptr(), n);
                }
                remaining = &remaining[n..];
            }
            let mut data_len = 1;
            DbnDecoder_data(decoder, &mut data_len);
            assert_eq!(data_len, 0);
            DbnDecoder_free(decoder);
        }

        assert_eq!(got_dataset.as_deref(), Some("GLBX.MDP3"));
        assert_eq!(decoded_ids, expected_ids);
    }

    #[test]
    fn data_reports_truncated_tail() {
        let (mut buffer, _) = sample_stream();
        buffer.truncate(buffer.len() - 4);
        let options = options(VersionUpgradePolicy::AsIs as u8, 0);
        unsafe {
            let decoder = DbnDecoder_create(&options, null_mut());
            assert!(!decoder.is_null());
            DbnDecoder_write_all(decoder, buffer.as_ptr(), buffer.len());
            loop {
                let mut read_more = 0;
                let mut metadata: *mut Metadata = null_mut();
                match DbnDecoder_process(decoder, &mut read_more, &mut metadata) {
                    ProcessStatus::ReadMore => break,
                    ProcessStatus::Metadata => drop(Box::from_raw(metadata)),
                    ProcessStatus::Record => {}
                    ProcessStatus::Error => panic!("decode error: unexpected"),
                }
            }
            let mut data_len = 0;
            DbnDecoder_data(decoder, &mut data_len);
            assert!(data_len > 0);
            DbnDecoder_free(decoder);
        }
    }

    #[rstest]
    #[case::null_options(None, DecoderError::NullOptions)]
    #[case::invalid_upgrade_policy(Some(options(9, 0)), DecoderError::InvalidUpgradePolicy)]
    #[case::invalid_input_version(
        Some(options(VersionUpgradePolicy::AsIs as u8, 9)),
        DecoderError::InvalidInputVersion
    )]
    #[case::incompatible_policy_and_version(
        Some(options(VersionUpgradePolicy::UpgradeToV2 as u8, 3)),
        DecoderError::IncompatiblePolicyAndVersion
    )]
    fn create_rejects_invalid_options(
        #[case] opts: Option<DecoderOptions>,
        #[case] expected: DecoderError,
    ) {
        let ptr = opts.as_ref().map_or(null(), |o| o as *const DecoderOptions);
        let mut error = DecoderError::NullOptions;
        let decoder = unsafe { DbnDecoder_create(ptr, &mut error) };
        assert!(decoder.is_null());
        assert_eq!(error, expected);
    }
}
