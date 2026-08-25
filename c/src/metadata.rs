//! The opaque DBN `Metadata` handle over the C FFI.

use std::ffi::c_char;

use dbn::{Metadata, UNDEF_TIMESTAMP};

use crate::encode::{build_metadata, write_error, EncodeError, MetadataRef};

/// A borrowed, non-null-terminated string: `data` points to `len` bytes valid
/// for the lifetime of the `Metadata` handle it came from.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct StrRef {
    pub data: *const c_char,
    pub len: usize,
}

impl StrRef {
    fn from_str(s: &str) -> Self {
        Self {
            data: s.as_ptr() as *const c_char,
            len: s.len(),
        }
    }

    pub(crate) fn null() -> Self {
        Self {
            data: std::ptr::null(),
            len: 0,
        }
    }
}

/// One symbol mapping interval. Dates are encoded as `YYYYMMDD` integers, as in
/// the DBN wire format.
#[repr(C)]
pub struct MappingIntervalRef {
    pub start_date: u32,
    pub end_date: u32,
    pub symbol: StrRef,
}

fn date_to_yyyymmdd(date: time::Date) -> u32 {
    date.year() as u32 * 10_000 + u8::from(date.month()) as u32 * 100 + date.day() as u32
}

/// The DBN version of the metadata.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_version(metadata: *const Metadata) -> u8 {
    metadata.as_ref().map(|m| m.version).unwrap_or(0)
}

/// The dataset code.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_dataset(metadata: *const Metadata) -> StrRef {
    metadata
        .as_ref()
        .map(|m| StrRef::from_str(&m.dataset))
        .unwrap_or_else(StrRef::null)
}

/// The record schema as a `Schema` discriminant. Returns `true` and sets
/// `schema` if one is present; returns `false` if the stream may contain
/// multiple record types.
///
/// # Safety
/// Verifies `metadata` and `schema` are not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_schema(metadata: *const Metadata, schema: *mut u16) -> bool {
    match (metadata.as_ref().and_then(|m| m.schema), schema.as_mut()) {
        (Some(value), Some(out)) => {
            *out = value as u16;
            true
        }
        _ => false,
    }
}

/// The UNIX nanosecond query start timestamp.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_start(metadata: *const Metadata) -> u64 {
    metadata.as_ref().map(|m| m.start).unwrap_or(0)
}

/// The UNIX nanosecond query end timestamp, or `UNDEF_TIMESTAMP` if unset.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_end(metadata: *const Metadata) -> u64 {
    metadata
        .as_ref()
        .and_then(|m| m.end)
        .map(|end| end.get())
        .unwrap_or(UNDEF_TIMESTAMP)
}

/// The maximum number of records for the query, or 0 if unset.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_limit(metadata: *const Metadata) -> u64 {
    metadata
        .as_ref()
        .and_then(|m| m.limit)
        .map(|limit| limit.get())
        .unwrap_or(0)
}

/// The input symbology type as an `SType` discriminant. Returns `true` and sets
/// `stype_in` if present; returns `false` for a mix (e.g. live data).
///
/// # Safety
/// Verifies `metadata` and `stype_in` are not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_stype_in(
    metadata: *const Metadata,
    stype_in: *mut u8,
) -> bool {
    match (
        metadata.as_ref().and_then(|m| m.stype_in),
        stype_in.as_mut(),
    ) {
        (Some(value), Some(out)) => {
            *out = value as u8;
            true
        }
        _ => false,
    }
}

/// The output symbology type as an `SType` discriminant.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_stype_out(metadata: *const Metadata) -> u8 {
    metadata.as_ref().map(|m| m.stype_out as u8).unwrap_or(0)
}

/// Whether records have send timestamps (`ts_out`) appended.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_ts_out(metadata: *const Metadata) -> bool {
    metadata.as_ref().is_some_and(|m| m.ts_out)
}

/// The length in bytes of fixed-length symbol strings, including the null
/// terminator.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_symbol_cstr_len(metadata: *const Metadata) -> usize {
    metadata.as_ref().map(|m| m.symbol_cstr_len).unwrap_or(0)
}

/// The number of requested symbols.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_symbols_count(metadata: *const Metadata) -> usize {
    metadata.as_ref().map(|m| m.symbols.len()).unwrap_or(0)
}

/// The requested symbol at `index`, or a null `StrRef` if out of bounds.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_symbols_get(
    metadata: *const Metadata,
    index: usize,
) -> StrRef {
    metadata
        .as_ref()
        .and_then(|m| m.symbols.get(index))
        .map(|s| StrRef::from_str(s))
        .unwrap_or_else(StrRef::null)
}

/// The number of symbols that couldn't be fully resolved.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_partial_count(metadata: *const Metadata) -> usize {
    metadata.as_ref().map(|m| m.partial.len()).unwrap_or(0)
}

/// The partially resolved symbol at `index`, or a null `StrRef` if out of
/// bounds.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_partial_get(
    metadata: *const Metadata,
    index: usize,
) -> StrRef {
    metadata
        .as_ref()
        .and_then(|m| m.partial.get(index))
        .map(|s| StrRef::from_str(s))
        .unwrap_or_else(StrRef::null)
}

/// The number of symbols that couldn't be resolved.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_not_found_count(metadata: *const Metadata) -> usize {
    metadata.as_ref().map(|m| m.not_found.len()).unwrap_or(0)
}

/// The unresolved symbol at `index`, or a null `StrRef` if out of bounds.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_not_found_get(
    metadata: *const Metadata,
    index: usize,
) -> StrRef {
    metadata
        .as_ref()
        .and_then(|m| m.not_found.get(index))
        .map(|s| StrRef::from_str(s))
        .unwrap_or_else(StrRef::null)
}

/// The number of symbol mappings.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_mappings_count(metadata: *const Metadata) -> usize {
    metadata.as_ref().map(|m| m.mappings.len()).unwrap_or(0)
}

/// The raw symbol of the mapping at `index`, or a null `StrRef` if out of
/// bounds.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_mapping_raw_symbol(
    metadata: *const Metadata,
    index: usize,
) -> StrRef {
    metadata
        .as_ref()
        .and_then(|m| m.mappings.get(index))
        .map(|mapping| StrRef::from_str(&mapping.raw_symbol))
        .unwrap_or_else(StrRef::null)
}

/// The number of intervals in the mapping at `index`.
///
/// # Safety
/// Verifies `metadata` is not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_mapping_intervals_count(
    metadata: *const Metadata,
    index: usize,
) -> usize {
    metadata
        .as_ref()
        .and_then(|m| m.mappings.get(index))
        .map(|mapping| mapping.intervals.len())
        .unwrap_or(0)
}

/// The interval `interval_index` of the mapping at `mapping_index`. Returns
/// `true` and sets `out` on success; returns `false` if out of bounds.
///
/// # Safety
/// Verifies `metadata` and `out` are not null.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_mapping_interval(
    metadata: *const Metadata,
    mapping_index: usize,
    interval_index: usize,
    out: *mut MappingIntervalRef,
) -> bool {
    let interval = metadata
        .as_ref()
        .and_then(|m| m.mappings.get(mapping_index))
        .and_then(|mapping| mapping.intervals.get(interval_index));
    match (interval, out.as_mut()) {
        (Some(interval), Some(out)) => {
            *out = MappingIntervalRef {
                start_date: date_to_yyyymmdd(interval.start_date),
                end_date: date_to_yyyymmdd(interval.end_date),
                symbol: StrRef::from_str(&interval.symbol),
            };
            true
        }
        _ => false,
    }
}

/// Converts a `DbnMetadataRef` into a handle for the `DbnMetadata_*` getters and
/// `DbnMetadata_encode`, or null on error, writing the reason to `error` when it is
/// non-null.
///
/// # Safety
/// `metadata` must be a valid pointer to a `DbnMetadataRef` whose own pointers are
/// valid for the duration of the call. `error`, if not null, must be a valid pointer.
/// The returned handle must be freed with `DbnMetadata_free`.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_from_ref(
    metadata: *const MetadataRef,
    error: *mut EncodeError,
) -> *mut Metadata {
    match build_metadata(metadata) {
        Ok(metadata) => Box::into_raw(Box::new(metadata)),
        Err(err) => {
            write_error(error, err);
            std::ptr::null_mut()
        }
    }
}

/// Frees a `Metadata` handle.
///
/// # Safety
/// `metadata` must have come from `DbnDecoder_process` or `DbnMetadata_from_ref` and
/// not been freed already. It must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn DbnMetadata_free(metadata: *mut Metadata) {
    if !metadata.is_null() {
        drop(Box::from_raw(metadata));
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use dbn::{
        enums::{SType, Schema},
        MappingInterval, SymbolMapping,
    };
    use time::macros::date;

    use super::*;
    use crate::test_utils::str_ref_to_string;

    #[test]
    fn free_accepts_null_and_owned_handles() {
        unsafe {
            DbnMetadata_free(std::ptr::null_mut());
            DbnMetadata_free(Box::into_raw(Box::new(
                Metadata::builder()
                    .dataset("XNAS.ITCH")
                    .schema(Some(Schema::Trades))
                    .stype_in(Some(SType::RawSymbol))
                    .stype_out(SType::InstrumentId)
                    .start(1)
                    .symbols(vec!["AAPL".to_owned()])
                    .build(),
            )));
        }
    }

    #[test]
    fn accessors_read_all_fields() {
        let owned = Metadata::builder()
            .dataset("XNAS.ITCH")
            .schema(Some(Schema::Trades))
            .start(1)
            .end(NonZeroU64::new(2))
            .limit(NonZeroU64::new(3))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .ts_out(true)
            .symbols(vec!["AAPL".to_owned()])
            .partial(vec!["MSFT".to_owned()])
            .not_found(vec!["TSLA".to_owned()])
            .mappings(vec![SymbolMapping {
                raw_symbol: "AAPL".to_owned(),
                intervals: vec![MappingInterval {
                    start_date: date!(2023 - 07 - 01),
                    end_date: date!(2023 - 08 - 01),
                    symbol: "32".to_owned(),
                }],
            }])
            .build();
        let metadata = &raw const owned;

        unsafe {
            assert_eq!(DbnMetadata_version(metadata), dbn::DBN_VERSION);
            assert_eq!(
                str_ref_to_string(DbnMetadata_dataset(metadata)),
                "XNAS.ITCH"
            );

            let mut schema = 0u16;
            assert!(DbnMetadata_schema(metadata, &mut schema));
            assert_eq!(schema, Schema::Trades as u16);

            assert_eq!(DbnMetadata_start(metadata), 1);
            assert_eq!(DbnMetadata_end(metadata), 2);
            assert_eq!(DbnMetadata_limit(metadata), 3);
            let mut stype_in = 0u8;
            assert!(DbnMetadata_stype_in(metadata, &mut stype_in));
            assert_eq!(stype_in, SType::RawSymbol as u8);
            assert_eq!(DbnMetadata_stype_out(metadata), SType::InstrumentId as u8);
            assert!(DbnMetadata_ts_out(metadata));
            assert_eq!(DbnMetadata_symbol_cstr_len(metadata), dbn::SYMBOL_CSTR_LEN);

            assert_eq!(DbnMetadata_symbols_count(metadata), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_symbols_get(metadata, 0)),
                "AAPL"
            );
            assert!(DbnMetadata_symbols_get(metadata, 1).data.is_null());

            assert_eq!(DbnMetadata_partial_count(metadata), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_partial_get(metadata, 0)),
                "MSFT"
            );
            assert_eq!(DbnMetadata_not_found_count(metadata), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_not_found_get(metadata, 0)),
                "TSLA"
            );

            assert_eq!(DbnMetadata_mappings_count(metadata), 1);
            assert_eq!(
                str_ref_to_string(DbnMetadata_mapping_raw_symbol(metadata, 0)),
                "AAPL"
            );
            assert_eq!(DbnMetadata_mapping_intervals_count(metadata, 0), 1);

            let mut interval = MappingIntervalRef {
                start_date: 0,
                end_date: 0,
                symbol: StrRef::null(),
            };
            assert!(DbnMetadata_mapping_interval(metadata, 0, 0, &mut interval));
            assert_eq!(interval.start_date, 20230701);
            assert_eq!(interval.end_date, 20230801);
            assert_eq!(str_ref_to_string(interval.symbol), "32");
            assert!(!DbnMetadata_mapping_interval(metadata, 0, 1, &mut interval));
        }
    }
}
