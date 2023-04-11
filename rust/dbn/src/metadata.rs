//! Contains [`Metadata`] struct which comes at the beginning of any DBN file or
//! stream and [`MetadataBuilder`] for creating a [`Metadata`] with defaults.
use std::num::NonZeroU64;

use serde::Serialize;

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
pub use dbn_macros::MockPyo3;

use crate::enums::{SType, Schema};
use crate::record::as_u8_slice;

/// Information about the data contained in a DBN file or stream. DBN requires the
/// Metadata to be included at the start of the encoded data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "python", pyo3::pyclass(module = "databento_dbn"))]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
pub struct Metadata {
    /// The DBN schema version number. Newly-encoded DBN files will use [`crate::DBN_VERSION`].
    #[pyo3(get)]
    pub version: u8,
    /// The dataset code.
    #[pyo3(get)]
    pub dataset: String,
    /// The data record schema. Specifies which record type is stored in the Zstd-compressed DBN file.
    #[pyo3(get)]
    pub schema: Schema,
    /// The UNIX nanosecond timestamp of the query start, or the first record if the file was split.
    #[pyo3(get)]
    pub start: u64,
    /// The UNIX nanosecond timestamp of the query end, or the last record if the file was split.
    #[pyo3(get)]
    pub end: Option<NonZeroU64>,
    /// The optional maximum number of records for the query.
    #[pyo3(get)]
    #[serde(serialize_with = "serialize_as_raw")]
    pub limit: Option<NonZeroU64>,
    /// The input symbology type to map from.
    #[pyo3(get)]
    pub stype_in: SType,
    /// The output symbology type to map to.
    #[pyo3(get)]
    pub stype_out: SType,
    /// `true` if this store contains live data with send timestamps appended to each
    /// record.
    #[pyo3(get)]
    pub ts_out: bool,
    /// The original query input symbols from the request.
    #[pyo3(get)]
    pub symbols: Vec<String>,
    /// Symbols that did not resolve for _at least one day_ in the query time range.
    #[pyo3(get)]
    pub partial: Vec<String>,
    /// Symbols that did not resolve for _any_ day in the query time range.
    #[pyo3(get)]
    pub not_found: Vec<String>,
    /// Symbol mappings containing a native symbol and its mapping intervals.
    pub mappings: Vec<SymbolMapping>,
}

/// Helper for constructing [`Metadata`] structs with defaults.
///
/// This struct uses type state to ensure at compile time that all the required fields
/// are set. If a required field is not set, `build()` won't be visible.
///
/// # Required fields
/// - [`dataset`](Metadata::dataset)
/// - [`schema`](Metadata::schema)
/// - [`start`](Metadata::start)
/// - [`stype_in`](Metadata::stype_in)
/// - [`stype_out`](Metadata::stype_out)
#[derive(Debug)]
pub struct MetadataBuilder<D, Sch, Start, StIn, StOut> {
    version: u8,
    dataset: D,
    schema: Sch,
    start: Start,
    end: Option<NonZeroU64>,
    limit: Option<NonZeroU64>,
    stype_in: StIn,
    stype_out: StOut,
    ts_out: bool,
    symbols: Vec<String>,
    partial: Vec<String>,
    not_found: Vec<String>,
    mappings: Vec<SymbolMapping>,
}

/// Sentinel type for a required field that has not yet been set.
pub struct Unset {}

impl MetadataBuilder<Unset, Unset, Unset, Unset, Unset> {
    /// Creates a new instance of the builder.
    pub fn new() -> Self {
        Self::default()
    }
}

impl AsRef<[u8]> for Metadata {
    fn as_ref(&self) -> &[u8] {
        unsafe { as_u8_slice(self) }
    }
}

impl<D, Sch, Start, StIn, StOut> MetadataBuilder<D, Sch, Start, StIn, StOut> {
    /// Sets the [`dataset`](Metadata::dataset) and returns the builder.
    pub fn dataset(self, dataset: String) -> MetadataBuilder<String, Sch, Start, StIn, StOut> {
        MetadataBuilder {
            version: self.version,
            dataset,
            schema: self.schema,
            start: self.start,
            end: self.end,
            limit: self.limit,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
            ts_out: self.ts_out,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
    }

    /// Sets the [`schema`](Metadata::schema) and returns the builder.
    pub fn schema(self, schema: Schema) -> MetadataBuilder<D, Schema, Start, StIn, StOut> {
        MetadataBuilder {
            version: self.version,
            dataset: self.dataset,
            schema,
            start: self.start,
            end: self.end,
            limit: self.limit,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
            ts_out: self.ts_out,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
    }

    /// Sets the [`start`](Metadata::start) and returns the builder.
    pub fn start(self, start: u64) -> MetadataBuilder<D, Sch, u64, StIn, StOut> {
        MetadataBuilder {
            version: self.version,
            dataset: self.dataset,
            schema: self.schema,
            start,
            end: self.end,
            limit: self.limit,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
            symbols: self.symbols,
            ts_out: self.ts_out,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
    }

    /// Sets the [`end`](Metadata::end) and returns the builder.
    pub fn end(mut self, end: Option<NonZeroU64>) -> Self {
        self.end = end;
        self
    }

    /// Sets the [`limit`](Metadata::limit) and returns the builder.
    pub fn limit(mut self, limit: Option<NonZeroU64>) -> Self {
        self.limit = limit;
        self
    }

    /// Sets the [`stype_in`](Metadata::stype_in) and returns the builder.
    pub fn stype_in(self, stype_in: SType) -> MetadataBuilder<D, Sch, Start, SType, StOut> {
        MetadataBuilder {
            version: self.version,
            dataset: self.dataset,
            schema: self.schema,
            start: self.start,
            end: self.end,
            limit: self.limit,
            stype_in,
            stype_out: self.stype_out,
            ts_out: self.ts_out,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
    }

    /// Sets the [`stype_out`](Metadata::stype_out) and returns the builder.
    pub fn stype_out(self, stype_out: SType) -> MetadataBuilder<D, Sch, Start, StIn, SType> {
        MetadataBuilder {
            version: self.version,
            dataset: self.dataset,
            schema: self.schema,
            start: self.start,
            end: self.end,
            limit: self.limit,
            stype_in: self.stype_in,
            stype_out,
            ts_out: self.ts_out,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
    }

    /// Sets the [`ts_out`](Metadata::ts_out) and returns the builder.
    pub fn ts_out(mut self, ts_out: bool) -> Self {
        self.ts_out = ts_out;
        self
    }

    /// Sets the [`symbols`](Metadata::symbols) and returns the builder.
    pub fn symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = symbols;
        self
    }

    /// Sets the [`partial`](Metadata::partial) and returns the builder.
    pub fn partial(mut self, partial: Vec<String>) -> Self {
        self.partial = partial;
        self
    }

    /// Sets the [`not_found`](Metadata::not_found) and returns the builder.
    pub fn not_found(mut self, not_found: Vec<String>) -> Self {
        self.not_found = not_found;
        self
    }

    /// Sets the [`mappings`](Metadata::mappings) and returns the builder.
    pub fn mappings(mut self, mappings: Vec<SymbolMapping>) -> Self {
        self.mappings = mappings;
        self
    }
}

impl MetadataBuilder<String, Schema, u64, SType, SType> {
    /// Constructs a [`Metadata`] object. The availability of this method indicates all
    /// required fields have been set.
    pub fn build(self) -> Metadata {
        Metadata {
            version: self.version,
            dataset: self.dataset,
            schema: self.schema,
            start: self.start,
            end: self.end,
            limit: self.limit,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
            ts_out: self.ts_out,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
    }
}

impl Default for MetadataBuilder<Unset, Unset, Unset, Unset, Unset> {
    fn default() -> Self {
        Self {
            version: crate::DBN_VERSION,
            dataset: Unset {},
            schema: Unset {},
            start: Unset {},
            end: None,
            limit: None,
            stype_in: Unset {},
            stype_out: Unset {},
            ts_out: false,
            symbols: vec![],
            partial: vec![],
            not_found: vec![],
            mappings: vec![],
        }
    }
}

/// A native symbol and its symbol mappings for different time ranges within the query range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "python", derive(pyo3::FromPyObject))]
pub struct SymbolMapping {
    /// The native symbol.
    pub native_symbol: String,
    /// The mappings of `native` for different date ranges.
    pub intervals: Vec<MappingInterval>,
}

/// The resolved symbol for a date range.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MappingInterval {
    /// The UTC start date of interval.
    #[serde(serialize_with = "serialize_date")]
    pub start_date: time::Date,
    /// The UTC end date of interval.
    #[serde(serialize_with = "serialize_date")]
    pub end_date: time::Date,
    /// The resolved symbol for this interval.
    pub symbol: String,
}

// Override `time::Date`'s serialization format to be ISO 8601.
fn serialize_date<S: serde::Serializer>(
    date: &time::Date,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&date.to_string()) // ISO 8601
}

fn serialize_as_raw<S: serde::Serializer>(
    val: &Option<NonZeroU64>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(val.map(|n| n.get()).unwrap_or(0))
}
