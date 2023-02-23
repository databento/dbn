//! Contains [`Metadata`] struct which comes at the beginning of any DBN file or
//! stream and [`MetadataBuilder`] for creating a [`Metadata`] with defaults.
use std::num::NonZeroU64;

use serde::Serialize;

use crate::enums::{rtype, SType, Schema};

/// Information about the data contained in a DBN file or stream. DBN requires the
/// Metadata to be included at the start of the encoded data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all))]
pub struct Metadata {
    /// The DBN schema version number. Newly-encoded DBN files will use [`crate::DBN_VERSION`].
    pub version: u8,
    /// The dataset code.
    pub dataset: String,
    /// The data record schema. Specifies which record type is stored in the Zstd-compressed DBN file.
    pub schema: Schema,
    /// The UNIX nanosecond timestamp of the query start, or the first record if the file was split.
    pub start: u64,
    /// The UNIX nanosecond timestamp of the query end, or the last record if the file was split.
    pub end: Option<NonZeroU64>,
    #[serde(serialize_with = "serialize_as_raw")]
    /// The optional maximum number of records for the query.
    pub limit: Option<NonZeroU64>,
    /// The total number of data records.
    pub record_count: Option<u64>,
    /// The input symbology type to map from.
    pub stype_in: SType,
    /// The output symbology type to map to.
    pub stype_out: SType,
    /// The original query input symbols from the request.
    pub symbols: Vec<String>,
    /// Symbols that did not resolve for _at least one day_ in the query time range.
    pub partial: Vec<String>,
    /// Symbols that did not resolve for _any_ day in the query time range.
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
    record_count: Option<u64>,
    stype_in: StIn,
    stype_out: StOut,
    symbols: Vec<String>,
    partial: Vec<String>,
    not_found: Vec<String>,
    mappings: Vec<SymbolMapping>,
}

impl Metadata {
    /// Returns the billable size of the stream being decoded if known. Billable
    /// size is the uncompressed size of all the records and excludes metadata.
    pub fn billable_size(&self) -> Option<usize> {
        match (
            rtype::record_size(rtype::from(self.schema)),
            self.record_count,
        ) {
            (Some(record_size), Some(record_count)) => Some(record_size * record_count as usize),
            _ => None,
        }
    }
}

/// Sentinel type for a required field that has not yet been set.
pub struct Unset {}

impl MetadataBuilder<Unset, Unset, Unset, Unset, Unset> {
    /// Creates a new instance of the builder.
    pub fn new() -> Self {
        Self::default()
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
            record_count: self.record_count,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
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
            record_count: self.record_count,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
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
            record_count: self.record_count,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
            symbols: self.symbols,
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

    /// Sets the [`record_count`](Metadata::record_count) and returns the builder.
    pub fn record_count(mut self, record_count: Option<u64>) -> Self {
        self.record_count = record_count;
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
            record_count: self.record_count,
            stype_in,
            stype_out: self.stype_out,
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
            record_count: self.record_count,
            stype_in: self.stype_in,
            stype_out,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
        }
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
            record_count: self.record_count,
            stype_in: self.stype_in,
            stype_out: self.stype_out,
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
            record_count: None,
            stype_in: Unset {},
            stype_out: Unset {},
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
    /// UTC start date of interval.
    #[serde(serialize_with = "serialize_date")]
    pub start_date: time::Date,
    /// UTC end date of interval.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_billable_size() {
        let target = MetadataBuilder::new()
            .schema(Schema::Mbp10)
            .dataset("XNAS.ITCH".to_owned())
            .start(1674856533000000000)
            .stype_in(SType::Native)
            .stype_out(SType::ProductId)
            .record_count(Some(100))
            .build();
        assert_eq!(
            target.billable_size(),
            Some(100 * rtype::record_size(rtype::MBP_10).unwrap())
        );
    }

    #[test]
    fn test_billable_size_none() {
        let target = MetadataBuilder::new()
            .schema(Schema::Mbp10)
            .dataset("XNAS.ITCH".to_owned())
            .start(1674856533000000000)
            .stype_in(SType::Native)
            .stype_out(SType::ProductId)
            .build();
        assert!(target.billable_size().is_none());
    }
}
