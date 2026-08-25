//! Encoding a DBN `Metadata` handle over the C FFI.

use std::{ffi::c_char, io, num::NonZeroU64, slice};

use dbn::{
    encode::dbn::MetadataEncoder,
    enums::{SType, Schema},
    MappingInterval, Metadata, MetadataBuilder, SymbolMapping, UNDEF_TIMESTAMP,
};

use crate::metadata::{MappingIntervalRef, StrRef};

/// The byte offset of the `start` field in DBN-encoded Metadata.
pub const METADATA_START_OFFSET: usize = 26;
/// The minimum buffer size in bytes for encoding DBN Metadata.
pub const METADATA_MIN_ENCODED_SIZE: usize = 128;
/// The value of `MetadataRef::schema` when the stream has no single schema.
pub const NULL_SCHEMA: u16 = u16::MAX;
/// The value of `MetadataRef::stype_in` when the stream has no single `stype_in`.
pub const NULL_STYPE: u8 = u8::MAX;

/// One symbol mapping: a raw symbol and the intervals over which it maps.
#[repr(C)]
pub struct SymbolMappingRef {
    pub raw_symbol: StrRef,
    pub intervals: *const MappingIntervalRef,
    pub intervals_len: usize,
}

/// DBN `Metadata` described with borrowed C data, for encoding. Every pointer is
/// borrowed for the duration of the call only.
///
/// `schema` and `stype_in` use the `NULL_SCHEMA` and `NULL_STYPE` wire sentinels
/// when absent, `end` uses `UNDEF_TIMESTAMP` and `limit` uses 0, matching what the
/// `DbnMetadata_*` accessors return.
#[repr(C)]
pub struct MetadataRef {
    pub version: u8,
    pub dataset: StrRef,
    pub schema: u16,
    pub start: u64,
    pub end: u64,
    pub limit: u64,
    pub stype_in: u8,
    pub stype_out: u8,
    pub ts_out: bool,
    pub symbols: *const StrRef,
    pub symbols_len: usize,
    pub partial: *const StrRef,
    pub partial_len: usize,
    pub not_found: *const StrRef,
    pub not_found_len: usize,
    pub mappings: *const SymbolMappingRef,
    pub mappings_len: usize,
}

/// The reason encoding failed.
#[repr(C)]
#[derive(Debug, PartialEq, Eq)]
pub enum EncodeError {
    /// The `metadata` pointer was null.
    NullMetadata,
    /// `dataset` was empty, null, or not UTF-8.
    InvalidDataset,
    /// A symbol or mapping string was null or not UTF-8, or a `symbols`, `partial`,
    /// `not_found`, `mappings`, or `intervals` array pointer was null despite a nonzero
    /// count.
    InvalidUtf8,
    /// `schema` was neither a valid `Schema` nor `NULL_SCHEMA`.
    InvalidSchema,
    /// `stype_in` or `stype_out` was not a valid `SType`.
    InvalidSType,
    /// `version` was 0 or newer than the supported DBN version.
    InvalidVersion,
    /// A mapping interval date was not a valid `YYYYMMDD` value.
    InvalidDate,
    /// `buffer` was null or shorter than `DbnMetadata_encoded_size` bytes.
    BufferTooSmall,
    /// Encoding failed for another reason.
    Encode,
}

/// The name of `error` as a static null-terminated string, e.g. `"InvalidDataset"`,
/// for logging.
#[no_mangle]
pub extern "C" fn DbnEncodeError_str(error: EncodeError) -> *const c_char {
    match error {
        EncodeError::NullMetadata => c"NullMetadata",
        EncodeError::InvalidDataset => c"InvalidDataset",
        EncodeError::InvalidUtf8 => c"InvalidUtf8",
        EncodeError::InvalidSchema => c"InvalidSchema",
        EncodeError::InvalidSType => c"InvalidSType",
        EncodeError::InvalidVersion => c"InvalidVersion",
        EncodeError::InvalidDate => c"InvalidDate",
        EncodeError::BufferTooSmall => c"BufferTooSmall",
        EncodeError::Encode => c"Encode",
    }
    .as_ptr()
}

pub(crate) unsafe fn write_error(error: *mut EncodeError, err: EncodeError) -> isize {
    if let Some(error) = error.as_mut() {
        *error = err;
    }
    -1
}

unsafe fn str_ref_to_owned(s: StrRef) -> Result<String, EncodeError> {
    if s.len == 0 {
        return Ok(String::new());
    }
    if s.data.is_null() {
        return Err(EncodeError::InvalidUtf8);
    }
    std::str::from_utf8(slice::from_raw_parts(s.data as *const u8, s.len))
        .map(str::to_owned)
        .map_err(|_| EncodeError::InvalidUtf8)
}

unsafe fn as_slice<'a, T>(ptr: *const T, len: usize) -> Result<&'a [T], EncodeError> {
    if len == 0 {
        Ok(&[])
    } else if ptr.is_null() {
        Err(EncodeError::InvalidUtf8)
    } else {
        Ok(slice::from_raw_parts(ptr, len))
    }
}

unsafe fn str_refs_to_owned(ptr: *const StrRef, len: usize) -> Result<Vec<String>, EncodeError> {
    as_slice(ptr, len)?
        .iter()
        .map(|s| str_ref_to_owned(*s))
        .collect()
}

unsafe fn mappings_to_owned(
    ptr: *const SymbolMappingRef,
    len: usize,
) -> Result<Vec<SymbolMapping>, EncodeError> {
    as_slice(ptr, len)?
        .iter()
        .map(|mapping| {
            let intervals = as_slice(mapping.intervals, mapping.intervals_len)?
                .iter()
                .map(|interval| {
                    Ok(MappingInterval {
                        start_date: yyyymmdd_to_date(interval.start_date)?,
                        end_date: yyyymmdd_to_date(interval.end_date)?,
                        symbol: str_ref_to_owned(interval.symbol)?,
                    })
                })
                .collect::<Result<Vec<_>, EncodeError>>()?;
            Ok(SymbolMapping {
                raw_symbol: str_ref_to_owned(mapping.raw_symbol)?,
                intervals,
            })
        })
        .collect()
}

pub(crate) unsafe fn build_metadata(metadata: *const MetadataRef) -> Result<Metadata, EncodeError> {
    let Some(metadata) = metadata.as_ref() else {
        return Err(EncodeError::NullMetadata);
    };
    if metadata.version == 0 || metadata.version > dbn::DBN_VERSION {
        return Err(EncodeError::InvalidVersion);
    }
    let dataset = str_ref_to_owned(metadata.dataset).map_err(|_| EncodeError::InvalidDataset)?;
    if dataset.is_empty() {
        return Err(EncodeError::InvalidDataset);
    }
    let schema = if metadata.schema == NULL_SCHEMA {
        None
    } else {
        Some(Schema::try_from(metadata.schema).map_err(|_| EncodeError::InvalidSchema)?)
    };
    let stype_in = if metadata.stype_in == NULL_STYPE {
        None
    } else {
        Some(SType::try_from(metadata.stype_in).map_err(|_| EncodeError::InvalidSType)?)
    };
    let stype_out = SType::try_from(metadata.stype_out).map_err(|_| EncodeError::InvalidSType)?;
    let end = if metadata.end == UNDEF_TIMESTAMP {
        None
    } else {
        NonZeroU64::new(metadata.end)
    };
    let built = MetadataBuilder::new()
        .version(metadata.version)
        .dataset(dataset)
        .schema(schema)
        .start(metadata.start)
        .end(end)
        .limit(NonZeroU64::new(metadata.limit))
        .stype_in(stype_in)
        .stype_out(stype_out)
        .ts_out(metadata.ts_out)
        .symbols(str_refs_to_owned(metadata.symbols, metadata.symbols_len)?)
        .partial(str_refs_to_owned(metadata.partial, metadata.partial_len)?)
        .not_found(str_refs_to_owned(
            metadata.not_found,
            metadata.not_found_len,
        )?)
        .mappings(mappings_to_owned(metadata.mappings, metadata.mappings_len)?)
        .build();
    Ok(built)
}

/// Returns the number of bytes `DbnMetadata_encode` will write for `metadata`, or -1 on
/// error, writing the reason to `error` when it is non-null.
///
/// # Safety
/// `metadata` must be a valid `DbnMetadata` handle. `error`, if not null, must be a
/// valid pointer.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_encoded_size(
    metadata: *const Metadata,
    error: *mut EncodeError,
) -> isize {
    let Some(metadata) = metadata.as_ref() else {
        return write_error(error, EncodeError::NullMetadata);
    };
    MetadataEncoder::<Vec<u8>>::encoded_len(metadata) as isize
}

/// Encodes `metadata` into `buffer`, returning the number of bytes written, or -1 on
/// error, writing the reason to `error` when it is non-null.
///
/// # Safety
/// `metadata` must be a valid `DbnMetadata` handle. `buffer` must be valid for
/// `capacity` bytes. `error`, if not null, must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_encode(
    metadata: *const Metadata,
    buffer: *mut u8,
    capacity: usize,
    error: *mut EncodeError,
) -> isize {
    let Some(metadata) = metadata.as_ref() else {
        return write_error(error, EncodeError::NullMetadata);
    };
    if buffer.is_null() || capacity < MetadataEncoder::<Vec<u8>>::encoded_len(metadata) {
        return write_error(error, EncodeError::BufferTooSmall);
    }
    let mut cursor = io::Cursor::new(slice::from_raw_parts_mut(buffer, capacity));
    match MetadataEncoder::new(&mut cursor).encode(metadata) {
        Ok(()) => cursor.position() as isize,
        Err(_) => write_error(error, EncodeError::Encode),
    }
}

fn yyyymmdd_to_date(yyyymmdd: u32) -> Result<time::Date, EncodeError> {
    let month = ((yyyymmdd / 100) % 100) as u8;
    time::Date::from_calendar_date(
        (yyyymmdd / 10_000) as i32,
        time::Month::try_from(month).map_err(|_| EncodeError::InvalidDate)?,
        (yyyymmdd % 100) as u8,
    )
    .map_err(|_| EncodeError::InvalidDate)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::{metadata::*, test_utils::str_ref_to_string};

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

    fn str_ref(s: &str) -> StrRef {
        StrRef {
            data: s.as_ptr() as *const c_char,
            len: s.len(),
        }
    }

    /// A `MetadataRef` with no collections, valid at `version`.
    fn scalar_ref(version: u8) -> MetadataRef {
        MetadataRef {
            version,
            dataset: str_ref("XNAS.ITCH"),
            schema: Schema::Trades as u16,
            start: 1,
            end: 2,
            limit: 3,
            stype_in: SType::RawSymbol as u8,
            stype_out: SType::InstrumentId as u8,
            ts_out: true,
            symbols: std::ptr::null(),
            symbols_len: 0,
            partial: std::ptr::null(),
            partial_len: 0,
            not_found: std::ptr::null(),
            not_found_len: 0,
            mappings: std::ptr::null(),
            mappings_len: 0,
        }
    }

    unsafe fn encode_exact(metadata: &MetadataRef) -> Vec<u8> {
        let mut error = EncodeError::Encode;
        let handle = DbnMetadata_from_ref(metadata, &mut error);
        assert!(!handle.is_null(), "conversion failed: {error:?}");

        let size = DbnMetadata_encoded_size(handle, &mut error);
        assert!(size > 0, "sizing failed: {error:?}");
        let size = size as usize;

        let mut buffer = vec![0xAAu8; size + 1];
        let written = DbnMetadata_encode(handle, buffer.as_mut_ptr(), size, &mut error);
        assert_eq!(written, size as isize, "encode failed: {error:?}");
        assert_eq!(buffer[size], 0xAA, "encode wrote past the reported size");
        DbnMetadata_free(handle);
        buffer.truncate(size);
        buffer
    }

    unsafe fn decode_metadata(bytes: &[u8]) -> *mut Metadata {
        let options = crate::decode::DecoderOptions {
            upgrade_policy: dbn::VersionUpgradePolicy::AsIs as u8,
            ts_out: false,
            input_version: 0,
            skip_metadata: false,
            buffer_size: 0,
        };
        let decoder = crate::decode::DbnDecoder_create(&options, std::ptr::null_mut());
        assert!(!decoder.is_null());
        crate::decode::DbnDecoder_write_all(decoder, bytes.as_ptr(), bytes.len());

        let mut read_more = 0;
        let mut decoded: *mut Metadata = std::ptr::null_mut();
        match crate::decode::DbnDecoder_process(decoder, &mut read_more, &mut decoded) {
            crate::decode::ProcessStatus::Metadata => {}
            crate::decode::ProcessStatus::ReadMore => panic!("ran out of bytes"),
            crate::decode::ProcessStatus::Record => panic!("record before metadata"),
            crate::decode::ProcessStatus::Error => panic!("decode error"),
        }
        crate::decode::DbnDecoder_free(decoder);
        assert!(!decoded.is_null());
        decoded
    }

    #[rstest]
    fn encode_round_trips_through_decoder(#[values(1, 2, 3)] version: u8) {
        let symbol_refs = [str_ref("AAPL"), str_ref("MSFT")];
        let partial_refs = [str_ref("TSLA")];
        let not_found_refs = [str_ref("NVDA")];
        let intervals = [
            MappingIntervalRef {
                start_date: 20230701,
                end_date: 20230801,
                symbol: str_ref("32"),
            },
            MappingIntervalRef {
                start_date: 20230801,
                end_date: 20230901,
                symbol: str_ref("33"),
            },
        ];
        let mappings = [SymbolMappingRef {
            raw_symbol: str_ref("AAPL"),
            intervals: intervals.as_ptr(),
            intervals_len: intervals.len(),
        }];

        let metadata = MetadataRef {
            symbols: symbol_refs.as_ptr(),
            symbols_len: symbol_refs.len(),
            partial: partial_refs.as_ptr(),
            partial_len: partial_refs.len(),
            not_found: not_found_refs.as_ptr(),
            not_found_len: not_found_refs.len(),
            mappings: mappings.as_ptr(),
            mappings_len: mappings.len(),
            ..scalar_ref(version)
        };

        unsafe {
            let encoded = encode_exact(&metadata);
            let decoded = decode_metadata(&encoded);

            assert_eq!(DbnMetadata_version(decoded), version);
            assert_eq!(str_ref_to_string(DbnMetadata_dataset(decoded)), "XNAS.ITCH");

            let mut schema = 0u16;
            assert!(DbnMetadata_schema(decoded, &mut schema));
            assert_eq!(schema, Schema::Trades as u16);

            assert_eq!(DbnMetadata_start(decoded), 1);
            assert_eq!(DbnMetadata_end(decoded), 2);
            assert_eq!(DbnMetadata_limit(decoded), 3);

            let mut stype_in = 0u8;
            assert!(DbnMetadata_stype_in(decoded, &mut stype_in));
            assert_eq!(stype_in, SType::RawSymbol as u8);
            assert_eq!(DbnMetadata_stype_out(decoded), SType::InstrumentId as u8);
            assert!(DbnMetadata_ts_out(decoded));
            assert_eq!(
                DbnMetadata_symbol_cstr_len(decoded),
                dbn::compat::version_symbol_cstr_len(version)
            );

            assert_eq!(DbnMetadata_symbols_count(decoded), 2);
            assert_eq!(
                str_ref_to_string(DbnMetadata_symbols_get(decoded, 0)),
                "AAPL"
            );
            assert_eq!(
                str_ref_to_string(DbnMetadata_symbols_get(decoded, 1)),
                "MSFT"
            );
            assert_eq!(DbnMetadata_partial_count(decoded), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_partial_get(decoded, 0)),
                "TSLA"
            );
            assert_eq!(DbnMetadata_not_found_count(decoded), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_not_found_get(decoded, 0)),
                "NVDA"
            );

            assert_eq!(DbnMetadata_mappings_count(decoded), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_mapping_raw_symbol(decoded, 0)),
                "AAPL"
            );
            assert_eq!(DbnMetadata_mapping_intervals_count(decoded, 0), 2);
            let mut interval = MappingIntervalRef {
                start_date: 0,
                end_date: 0,
                symbol: StrRef::null(),
            };
            assert!(DbnMetadata_mapping_interval(decoded, 0, 1, &mut interval));
            assert_eq!(interval.start_date, 20230801);
            assert_eq!(interval.end_date, 20230901);
            assert_eq!(str_ref_to_string(interval.symbol), "33");

            DbnMetadata_free(decoded);
        }
    }

    #[rstest]
    fn unset_end_and_limit_round_trip(#[values(1, 2, 3)] version: u8) {
        let metadata = MetadataRef {
            end: UNDEF_TIMESTAMP,
            limit: 0,
            schema: NULL_SCHEMA,
            stype_in: NULL_STYPE,
            ..scalar_ref(version)
        };
        unsafe {
            let decoded = decode_metadata(&encode_exact(&metadata));
            assert_eq!(DbnMetadata_end(decoded), UNDEF_TIMESTAMP);
            assert_eq!(DbnMetadata_limit(decoded), 0);
            assert!(!DbnMetadata_schema(decoded, &mut 0u16));
            assert!(!DbnMetadata_stype_in(decoded, &mut 0u8));
            DbnMetadata_free(decoded);
        }
    }

    #[rstest]
    fn encode_rejects_a_buffer_below_the_reported_size(#[values(1, 2, 3)] version: u8) {
        let metadata = scalar_ref(version);
        unsafe {
            let mut error = EncodeError::Encode;
            let handle = DbnMetadata_from_ref(&metadata, &mut error);
            let size = DbnMetadata_encoded_size(handle, &mut error) as usize;
            assert!(size >= METADATA_MIN_ENCODED_SIZE);

            let mut buffer = vec![0xAAu8; size];
            assert_eq!(
                DbnMetadata_encode(handle, buffer.as_mut_ptr(), size - 1, &mut error),
                -1
            );
            assert_eq!(error, EncodeError::BufferTooSmall);
            assert!(
                buffer.iter().all(|b| *b == 0xAA),
                "a rejected encode wrote into the buffer"
            );

            assert_eq!(
                DbnMetadata_encode(handle, std::ptr::null_mut(), size, &mut error),
                -1
            );
            assert_eq!(error, EncodeError::BufferTooSmall);
            DbnMetadata_free(handle);
        }
    }

    #[test]
    fn a_zero_initialized_metadata_ref_is_rejected() {
        // `MetadataRef` has no valid zero value, so the caller must not get one by
        // accident.
        let metadata: MetadataRef = unsafe { std::mem::zeroed() };
        let mut error = EncodeError::Encode;
        assert!(unsafe { DbnMetadata_from_ref(&metadata, &mut error) }.is_null());
        assert_eq!(error, EncodeError::InvalidVersion);
    }

    #[rstest]
    #[case::null_metadata(None, EncodeError::NullMetadata)]
    #[case::version_zero(Some(MetadataRef { version: 0, ..scalar_ref(3) }), EncodeError::InvalidVersion)]
    #[case::version_too_new(
        Some(MetadataRef { version: dbn::DBN_VERSION + 1, ..scalar_ref(3) }),
        EncodeError::InvalidVersion
    )]
    #[case::empty_dataset(
        Some(MetadataRef { dataset: StrRef::null(), ..scalar_ref(3) }),
        EncodeError::InvalidDataset
    )]
    #[case::bad_schema(
        Some(MetadataRef { schema: 9_999, ..scalar_ref(3) }),
        EncodeError::InvalidSchema
    )]
    #[case::bad_stype_out(
        Some(MetadataRef { stype_out: 200, ..scalar_ref(3) }),
        EncodeError::InvalidSType
    )]
    fn from_ref_rejects_invalid_metadata(
        #[case] metadata: Option<MetadataRef>,
        #[case] expected: EncodeError,
    ) {
        let ptr = metadata
            .as_ref()
            .map(|m| m as *const _)
            .unwrap_or_else(std::ptr::null);
        let mut error = EncodeError::Encode;
        assert!(unsafe { DbnMetadata_from_ref(ptr, &mut error) }.is_null());
        assert_eq!(error, expected);
    }

    #[test]
    fn a_null_handle_is_rejected() {
        let mut error = EncodeError::Encode;
        assert_eq!(
            unsafe { DbnMetadata_encoded_size(std::ptr::null(), &mut error) },
            -1
        );
        assert_eq!(error, EncodeError::NullMetadata);

        let mut error = EncodeError::Encode;
        let mut buffer = [0u8; 4096];
        assert_eq!(
            unsafe {
                DbnMetadata_encode(
                    std::ptr::null(),
                    buffer.as_mut_ptr(),
                    buffer.len(),
                    &mut error,
                )
            },
            -1
        );
        assert_eq!(error, EncodeError::NullMetadata);
    }

    #[test]
    fn encode_rejects_an_oversized_symbol() {
        let long = "A".repeat(dbn::SYMBOL_CSTR_LEN);
        let symbols = [str_ref(&long)];
        let metadata = MetadataRef {
            symbols: symbols.as_ptr(),
            symbols_len: symbols.len(),
            ..scalar_ref(3)
        };
        unsafe {
            let mut error = EncodeError::NullMetadata;
            let handle = DbnMetadata_from_ref(&metadata, &mut error);
            assert!(!handle.is_null());
            // Sizing is arithmetic over the collections, so only encoding sees this.
            assert!(DbnMetadata_encoded_size(handle, &mut error) > 0);

            let mut buffer = [0u8; 4096];
            assert_eq!(
                DbnMetadata_encode(handle, buffer.as_mut_ptr(), buffer.len(), &mut error),
                -1
            );
            assert_eq!(error, EncodeError::Encode);
            DbnMetadata_free(handle);
        }
    }

    #[test]
    fn from_ref_rejects_a_bad_mapping_date() {
        let intervals = [MappingIntervalRef {
            start_date: 20231301,
            end_date: 20230801,
            symbol: str_ref("32"),
        }];
        let mappings = [SymbolMappingRef {
            raw_symbol: str_ref("AAPL"),
            intervals: intervals.as_ptr(),
            intervals_len: intervals.len(),
        }];
        let metadata = MetadataRef {
            mappings: mappings.as_ptr(),
            mappings_len: mappings.len(),
            ..scalar_ref(3)
        };
        let mut error = EncodeError::Encode;
        assert!(unsafe { DbnMetadata_from_ref(&metadata, &mut error) }.is_null());
        assert_eq!(error, EncodeError::InvalidDate);
    }

    #[test]
    fn unset_end_and_limit_use_their_wire_sentinels() {
        let owned = Metadata::builder()
            .dataset("XNAS.ITCH")
            .schema(Some(Schema::Trades))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .start(1)
            .end(None)
            .limit(None)
            .build();
        let metadata = &raw const owned;

        unsafe {
            // The sentinels differ between the two fields, so each getter returns
            // what an encoder would write rather than a shared "unset" value.
            assert_eq!(DbnMetadata_end(metadata), UNDEF_TIMESTAMP);
            assert_eq!(DbnMetadata_limit(metadata), 0);
        }
    }
}
