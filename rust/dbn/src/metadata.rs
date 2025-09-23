//! Contains [`Metadata`] struct which comes at the beginning of any DBN file or
//! stream and [`MetadataBuilder`] for creating a [`Metadata`] with defaults.

mod merge;

use std::num::NonZeroU64;

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

use merge::MetadataMerger;
#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{
    compat::version_symbol_cstr_len, record::as_u8_slice, PitSymbolMap, SType, Schema, TsSymbolMap,
    VersionUpgradePolicy,
};

/// Information about the data contained in a DBN file or stream. DBN requires the
/// Metadata to be included at the start of the encoded data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyo3::pyclass(eq, module = "databento_dbn"))]
#[cfg_attr(not(feature = "python"), derive(MockPyo3))] // bring `pyo3` attribute into scope
pub struct Metadata {
    /// The DBN schema version number. Newly-encoded DBN files will use
    /// [`crate::DBN_VERSION`].
    #[pyo3(get)]
    pub version: u8,
    /// The dataset code.
    #[pyo3(get)]
    pub dataset: String,
    /// The data record schema. Specifies which record types are in the DBN stream.
    /// `None` indicates the DBN stream _may_ contain more than one record type.
    #[pyo3(get)]
    pub schema: Option<Schema>,
    /// The UNIX nanosecond timestamp of the query start, or the first record if the
    /// file was split.
    #[pyo3(get)]
    pub start: u64,
    /// The UNIX nanosecond timestamp of the query end, or the last record if the file
    /// was split.
    #[pyo3(get)]
    pub end: Option<NonZeroU64>,
    /// The optional maximum number of records for the query.
    #[pyo3(get)]
    pub limit: Option<NonZeroU64>,
    /// The input symbology type to map from. `None` indicates a mix, such as in the
    /// case of live data.
    #[pyo3(get)]
    pub stype_in: Option<SType>,
    /// The output symbology type to map to.
    #[pyo3(get)]
    pub stype_out: SType,
    /// `true` if this store contains live data with send timestamps appended to each
    /// record.
    #[pyo3(get)]
    pub ts_out: bool,
    /// The length in bytes of fixed-length symbol strings, including a null terminator
    /// byte.
    #[pyo3(get)]
    pub symbol_cstr_len: usize,
    /// The original query input symbols from the request.
    #[pyo3(get)]
    pub symbols: Vec<String>,
    /// Symbols that did not resolve for _at least one day_ in the query time range.
    #[pyo3(get)]
    pub partial: Vec<String>,
    /// Symbols that did not resolve for _any_ day in the query time range.
    #[pyo3(get)]
    pub not_found: Vec<String>,
    /// Symbol mappings containing a raw symbol and its mapping intervals.
    pub mappings: Vec<SymbolMapping>,
}

impl Metadata {
    /// Creates a builder for building `Metadata`. Call `.dataset(...)`, `.schema(...)`,
    /// `.start(...)` `.stype_in(...)`, and `.stype_out(...)` on the builder to set the
    /// required fields. Finally call `.build()` to create the `Metadata` instance.
    pub fn builder() -> MetadataBuilder<Unset, Unset, Unset, Unset, Unset> {
        MetadataBuilder::default()
    }

    /// Parses the raw query start into a datetime.
    pub fn start(&self) -> time::OffsetDateTime {
        // `u64::MAX` is within the allowable range for `OffsetDateTime`s
        time::OffsetDateTime::from_unix_timestamp_nanos(self.start as i128).unwrap()
    }

    /// Parses the raw query end time or the timestamp of the last record into a
    /// datetime. Returns `None` if  the end time was not specified.
    pub fn end(&self) -> Option<time::OffsetDateTime> {
        self.end
            .map(|end| time::OffsetDateTime::from_unix_timestamp_nanos(end.get() as i128).unwrap())
    }

    /// Creates a symbology mapping from instrument ID to text symbol for the given
    /// date.
    ///
    /// This method is useful when working with a historical request over a single day
    /// or in other situations where you're sure the mappings don't change during the
    /// time range of the request. Otherwise, [`Self::symbol_map()`] is recommended.
    ///
    /// # Errors
    /// This function returns an error if `stype_out` is not [`SType::InstrumentId`] or
    /// it can't parse a symbol into a `u32` instrument ID. It will also return an error
    /// if `date` is outside the query range.
    pub fn symbol_map_for_date(&self, date: time::Date) -> crate::Result<PitSymbolMap> {
        PitSymbolMap::from_metadata(self, date)
    }

    /// Creates a symbology mapping from instrument ID and date to text symbol.
    ///
    /// If you're working with a single date or otherwise don't expect the mappings to
    /// change, [`Self::symbol_map_for_date()`] is recommended.
    ///
    /// # Errors
    /// This function returns an error if `stype_out` is not [`SType::InstrumentId`] or
    /// it can't parse a symbol into a `u32` instrument ID.
    pub fn symbol_map(&self) -> crate::Result<TsSymbolMap> {
        TsSymbolMap::from_metadata(self)
    }

    /// Upgrades the metadata according to `upgrade_policy` if necessary.
    pub fn upgrade(&mut self, upgrade_policy: VersionUpgradePolicy) {
        if self.version < 2 {
            match upgrade_policy {
                VersionUpgradePolicy::AsIs => {
                    self.symbol_cstr_len = crate::v1::SYMBOL_CSTR_LEN;
                }
                VersionUpgradePolicy::UpgradeToV2 => {
                    self.version = 2;
                    self.symbol_cstr_len = crate::v2::SYMBOL_CSTR_LEN;
                }
                VersionUpgradePolicy::UpgradeToV3 => {
                    self.version = 3;
                    self.symbol_cstr_len = crate::v3::SYMBOL_CSTR_LEN;
                }
            }
        } else if self.version == 2 && upgrade_policy == VersionUpgradePolicy::UpgradeToV3 {
            self.version = 3;
        }
    }

    /// Attempts to merge another metadata into this one. This is useful for merging
    /// DBN streams.
    ///
    /// If merging data from multiple schemas, the resulting metadata will have a schema
    /// of `None`.
    ///
    /// # Errors
    /// Merging metadata where any of the following fields don't match will result in
    /// an error:
    /// - `version`: upgrade the metadata of the lower version before merging
    /// - `dataset`
    /// - `stype_in`
    /// - `stype_out`
    /// - `ts_out`
    /// - `symbol_cstr_len`: upgrade the metadata of the lower version before merging
    ///
    /// This function will also return an error if there are conflicting symbology
    /// mappings.
    pub fn merge(self, other: impl IntoIterator<Item = Metadata>) -> crate::Result<Self> {
        let mut merger = MetadataMerger::new(self);
        for metadata in other {
            merger.merge(metadata)?;
        }
        Ok(merger.finalize())
    }
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
    /// Sets [`version`](Metadata::version) and returns the builder.
    pub fn version(mut self, version: u8) -> Self {
        self.version = version;
        self
    }

    /// Sets [`dataset`](Metadata::dataset) and returns the builder.
    pub fn dataset(
        self,
        dataset: impl ToString,
    ) -> MetadataBuilder<String, Sch, Start, StIn, StOut> {
        MetadataBuilder {
            version: self.version,
            dataset: dataset.to_string(),
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

    /// Sets [`schema`](Metadata::schema) and returns the builder.
    pub fn schema(
        self,
        schema: Option<Schema>,
    ) -> MetadataBuilder<D, Option<Schema>, Start, StIn, StOut> {
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

    /// Sets [`start`](Metadata::start) and returns the builder.
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

    /// Sets [`end`](Metadata::end) and returns the builder.
    pub fn end(mut self, end: Option<NonZeroU64>) -> Self {
        self.end = end;
        self
    }

    /// Sets [`limit`](Metadata::limit) and returns the builder.
    pub fn limit(mut self, limit: Option<NonZeroU64>) -> Self {
        self.limit = limit;
        self
    }

    /// Sets [`stype_in`](Metadata::stype_in) and returns the builder.
    pub fn stype_in(
        self,
        stype_in: Option<SType>,
    ) -> MetadataBuilder<D, Sch, Start, Option<SType>, StOut> {
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

    /// Sets [`stype_out`](Metadata::stype_out) and returns the builder.
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

    /// Sets [`ts_out`](Metadata::ts_out) and returns the builder.
    pub fn ts_out(mut self, ts_out: bool) -> Self {
        self.ts_out = ts_out;
        self
    }

    /// Sets [`symbols`](Metadata::symbols) and returns the builder.
    pub fn symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = symbols;
        self
    }

    /// Sets [`partial`](Metadata::partial) and returns the builder.
    pub fn partial(mut self, partial: Vec<String>) -> Self {
        self.partial = partial;
        self
    }

    /// Sets [`not_found`](Metadata::not_found) and returns the builder.
    pub fn not_found(mut self, not_found: Vec<String>) -> Self {
        self.not_found = not_found;
        self
    }

    /// Sets [`mappings`](Metadata::mappings) and returns the builder.
    pub fn mappings(mut self, mappings: Vec<SymbolMapping>) -> Self {
        self.mappings = mappings;
        self
    }
}

impl MetadataBuilder<String, Option<Schema>, u64, Option<SType>, SType> {
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
            symbol_cstr_len: version_symbol_cstr_len(self.version),
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

/// A raw symbol and its symbol mappings for different time ranges within the query range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[cfg_attr(feature = "python", derive(pyo3::FromPyObject))]
pub struct SymbolMapping {
    /// The `stype_in` symbol.
    pub raw_symbol: String,
    /// The mappings of `raw_symbol` to `stype_out` for different date ranges.
    pub intervals: Vec<MappingInterval>,
}

/// The resolved symbol for a date range.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub struct MappingInterval {
    /// The UTC start date of interval (inclusive).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "d0", deserialize_with = "deserialize_date")
    )]
    pub start_date: time::Date,
    /// The UTC end date of interval (exclusive).
    #[cfg_attr(
        feature = "serde",
        serde(rename = "d1", deserialize_with = "deserialize_date")
    )]
    pub end_date: time::Date,
    /// The resolved symbol for this interval (in `stype_out`).
    #[cfg_attr(feature = "serde", serde(rename = "s"))]
    pub symbol: String,
}

/// The date format used for date strings when serializing [`Metadata`].
pub const DATE_FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");

#[cfg(feature = "serde")]
fn deserialize_date<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<time::Date, D::Error> {
    let date_str = String::deserialize(deserializer)?;
    time::Date::parse(&date_str, DATE_FORMAT).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::Dataset;

    use super::*;

    #[rstest]
    #[case(VersionUpgradePolicy::AsIs, 1)]
    #[case(VersionUpgradePolicy::UpgradeToV2, 2)]
    #[case(VersionUpgradePolicy::UpgradeToV3, 3)]
    fn test_upgrade_metadata(
        #[case] upgrade_policy: VersionUpgradePolicy,
        #[case] exp_version: u8,
    ) {
        let mut target = Metadata::builder()
            .version(1)
            .dataset(Dataset::OpraPillar)
            .schema(Some(Schema::Mbp1))
            .start(0)
            .stype_in(None)
            .stype_out(SType::InstrumentId)
            .build();
        assert_eq!(target.version, 1);
        target.upgrade(upgrade_policy);
        assert_eq!(target.version, exp_version);
    }
}
