//! Contains [`Metadata`] struct which comes at the beginning of any DBN file or
//! stream and [`MetadataBuilder`] for creating a [`Metadata`] with defaults.

mod lengths;
mod merge;

use std::{mem, num::NonZeroU64};

pub(crate) use lengths::MetadataLengths;

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
use dbn_macros::MockPyo3;

use merge::MetadataMerger;
#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::{
    compat::version_symbol_cstr_len,
    layout::{LayoutIndex, StreamLayout},
    record::as_u8_slice,
    symbol_map::date_to_ts,
    PitSymbolMap, SType, Schema, TsSymbolMap, VersionUpgradePolicy,
};

/// Information about the data contained in a DBN file or stream. DBN requires the
/// Metadata to be included at the start of the encoded data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "python",
    pyo3::pyclass(eq, from_py_object, module = "databento_dbn")
)]
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
    ///
    /// In version 4, this field is superseded by whether `layout` specifies a `ts_out`
    /// extension field.
    #[pyo3(get)]
    pub ts_out: bool,
    /// The length in bytes of fixed-length symbol strings, including a null terminator
    /// byte.
    #[pyo3(get)]
    pub symbol_cstr_len: usize,
    /// The layout of every record type in the stream. `None` for DBN versions 1-3.
    pub layout: Option<StreamLayout>,
    /// The original query input symbols from the request.
    #[pyo3(get)]
    pub symbols: Vec<String>,
    /// Symbols that did not resolve for _at least one day_ in the query time range.
    #[pyo3(get)]
    pub partial: Vec<String>,
    /// Symbols that did not resolve for _any_ day in the query time range.
    #[pyo3(get)]
    pub not_found: Vec<String>,
    /// Symbol mappings containing a raw symbol and its mapping intervals. Only used in
    /// DBN versions 1-3.
    pub mappings: Vec<SymbolMapping>,
    /// Symbol mappings with nanosecond-precision intervals. Only used in DBN version 4.
    pub intervals: Vec<SymbolInterval>,
}

impl Metadata {
    /// Creates a builder for building `Metadata`. Call `.dataset(...)`, `.schema(...)`,
    /// `.start(...)` `.stype_in(...)`, and `.stype_out(...)` on the builder to set the
    /// required fields. Finally call `.build()` to create the `Metadata` instance.
    pub fn builder() -> MetadataBuilder<Unset, Unset, Unset, Unset, Unset> {
        MetadataBuilder::default()
    }

    /// An index over this stream's layout, for resolving fields the compiled structs
    /// don't hold. `None` for DBN versions 1-3, which carry no layout.
    pub fn layout_index(&self) -> Option<LayoutIndex> {
        self.layout
            .as_ref()
            .map(|layout| LayoutIndex::new(layout, crate::v4::layouts::default_layout()))
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
    /// This function returns an error if neither `stype_in` nor `stype_out` is
    /// [`SType::InstrumentId`], or if a symbol cannot be parsed into a `u32` instrument
    /// ID, or if `date` is outside the query range.
    pub fn symbol_map_for_date(&self, date: time::Date) -> crate::Result<PitSymbolMap> {
        PitSymbolMap::from_metadata(self, date)
    }

    /// Creates a symbology mapping from instrument ID and date to text symbol.
    ///
    /// If you're working with a single date or otherwise don't expect the mappings to
    /// change, [`Self::symbol_map_for_date()`] is recommended.
    ///
    /// # Errors
    /// This function returns an error if neither `stype_in` nor `stype_out` is
    /// [`SType::InstrumentId`], or if a symbol cannot be parsed into a `u32` instrument
    /// ID.
    pub fn symbol_map(&self) -> crate::Result<TsSymbolMap> {
        TsSymbolMap::from_metadata(self)
    }

    /// Upgrades the metadata according to `upgrade_policy` if necessary.
    pub fn upgrade(&mut self, upgrade_policy: VersionUpgradePolicy) {
        let target = if upgrade_policy.is_upgrade_situation(self.version) {
            upgrade_policy.output_version(self.version)
        } else {
            self.version
        };
        if self.version < 2 {
            self.symbol_cstr_len = version_symbol_cstr_len(target);
        }
        if target > self.version && target >= crate::v4::DBN_VERSION && self.layout.is_none() {
            self.layout = Some(crate::layout::upgrade_layout(self.ts_out));
        }
        if target >= crate::v4::DBN_VERSION {
            self.upgrade_mappings();
        }
        self.version = target;
    }

    /// Converts version 1-3 `mappings` into version 4 `intervals`. Where neither `stype_in`
    /// nor `stype_out` is [`SType::InstrumentId`], `instrument_id` is `u64::MAX`.
    fn upgrade_mappings(&mut self) {
        let is_inverse = self.is_inverse().ok();
        for mapping in mem::take(&mut self.mappings) {
            for interval in mapping.intervals {
                // handle old symbology format
                if interval.symbol.is_empty() {
                    continue;
                }
                let instrument_id = match is_inverse {
                    Some(false) => interval.symbol.parse().ok(),
                    Some(true) => mapping.raw_symbol.parse().ok(),
                    None => None,
                };
                self.intervals.push(SymbolInterval {
                    start_ts: date_to_ts(interval.start_date),
                    end_ts: date_to_ts(interval.end_date),
                    instrument_id: instrument_id.unwrap_or(u64::MAX),
                    stype_in_symbol: mapping.raw_symbol.clone(),
                    stype_out_symbol: interval.symbol,
                });
            }
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

    /// Returns `true` if the metadata is for inverse mappings, where `stype_in` is
    /// [`SType::InstrumentId`].
    ///
    /// # Errors
    /// This function returns an error if neither `stype_in` and `stype_out` are
    /// [`SType::InstrumentId`].
    pub fn is_inverse(&self) -> crate::Result<bool> {
        match (self.stype_in, self.stype_out) {
        (_, SType::InstrumentId) => Ok(false),
        (Some(SType::InstrumentId), _) => Ok(true),
        _ => {
            Err(crate::Error::BadArgument {
                param_name: "self".to_owned(),
                desc: "Can only create symbol maps from metadata where either stype_out or stype_in is instrument ID".to_owned(),
            })
        }
    }
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
    layout: Option<StreamLayout>,
    symbols: Vec<String>,
    partial: Vec<String>,
    not_found: Vec<String>,
    mappings: Vec<SymbolMapping>,
    intervals: Vec<SymbolInterval>,
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
            layout: self.layout,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
            intervals: self.intervals,
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
            layout: self.layout,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
            intervals: self.intervals,
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
            layout: self.layout,
            symbols: self.symbols,
            ts_out: self.ts_out,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
            intervals: self.intervals,
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
            layout: self.layout,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
            intervals: self.intervals,
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
            layout: self.layout,
            symbols: self.symbols,
            partial: self.partial,
            not_found: self.not_found,
            mappings: self.mappings,
            intervals: self.intervals,
        }
    }

    /// Sets [`ts_out`](Metadata::ts_out) and returns the builder.
    pub fn ts_out(mut self, ts_out: bool) -> Self {
        self.ts_out = ts_out;
        self
    }

    /// Sets the [`layout`](Metadata::layout) and returns the builder.
    pub fn layout(mut self, layout: StreamLayout) -> Self {
        self.layout = Some(layout);
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

    /// Sets [`intervals`](Metadata::intervals) and returns the builder.
    pub fn intervals(mut self, intervals: Vec<SymbolInterval>) -> Self {
        self.intervals = intervals;
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
            intervals: self.intervals,
            symbol_cstr_len: version_symbol_cstr_len(self.version),
            // TODO(cg): look at making this method fallible and validating v4 metadata
            // has non-None `layout`. Currently validated in `MetadataEncoder`
            layout: self.layout,
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
            layout: None,
            symbols: vec![],
            partial: vec![],
            not_found: vec![],
            mappings: vec![],
            intervals: vec![],
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

/// A symbol mapping over a nanosecond-precision time range. The form of symbology in
/// DBN version 4.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SymbolInterval {
    /// The start of the interval (inclusive) expressed as the number of nanoseconds
    /// since the UNIX epoch.
    pub start_ts: u64,
    /// The end of the interval (exclusive) expressed as the number of nanoseconds since
    /// the UNIX epoch. [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP) if open-ended.
    pub end_ts: u64,
    /// The numeric instrument ID.
    pub instrument_id: u64,
    /// The symbol in `stype_in`.
    pub stype_in_symbol: String,
    /// The symbol in `stype_out`.
    pub stype_out_symbol: String,
}

/// The date format used for date strings when serializing [`Metadata`].
pub const DATE_FORMAT: &[time::format_description::BorrowedFormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");

/// Encodes `date` as a `YYYYMMDD` integer, the form used in DBN metadata
/// symbol-mapping intervals (see [`MappingInterval`]) and in the JSON/parquet
/// metadata representations.
pub(crate) fn date_to_yyyymmdd(date: time::Date) -> u32 {
    date.year() as u32 * 10_000 + date.month() as u32 * 100 + date.day() as u32
}

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

    fn v1_metadata() -> Metadata {
        Metadata::builder()
            .version(1)
            .dataset(Dataset::OpraPillar)
            .schema(Some(Schema::Mbp1))
            .start(0)
            .stype_in(None)
            .stype_out(SType::InstrumentId)
            .build()
    }

    #[rstest]
    #[case(VersionUpgradePolicy::AsIs, 1)]
    #[case(VersionUpgradePolicy::UpgradeToV2, 2)]
    #[case(VersionUpgradePolicy::UpgradeToV3, 3)]
    #[case(VersionUpgradePolicy::UpgradeToV4, 4)]
    #[case(VersionUpgradePolicy::ExpectV4, 1)]
    fn test_upgrade_metadata(
        #[case] upgrade_policy: VersionUpgradePolicy,
        #[case] exp_version: u8,
    ) {
        let mut target = v1_metadata();
        assert_eq!(target.version, 1);
        target.upgrade(upgrade_policy);
        assert_eq!(target.version, exp_version);
    }

    #[test]
    fn test_upgrade_to_v4_attaches_the_upgrade_layout() {
        let mut target = v1_metadata();
        assert!(target.layout.is_none());

        target.upgrade(VersionUpgradePolicy::UpgradeToV4);
        assert_eq!(
            target.layout.as_ref(),
            Some(&crate::layout::upgrade_layout(false))
        );
        assert_eq!(target.symbol_cstr_len, crate::v3::SYMBOL_CSTR_LEN);
    }

    #[test]
    fn test_upgrade_to_v4_without_instrument_ids() {
        let mut target = Metadata::builder()
            .version(3)
            .dataset(Dataset::GlbxMdp3)
            .schema(Some(Schema::Trades))
            .start(0)
            .stype_in(Some(SType::Continuous))
            .stype_out(SType::RawSymbol)
            .mappings(vec![SymbolMapping {
                raw_symbol: "ES.c.0".to_owned(),
                intervals: vec![MappingInterval {
                    start_date: time::macros::date!(2023 - 12 - 01),
                    end_date: time::macros::date!(2023 - 12 - 02),
                    symbol: "ESZ3".to_owned(),
                }],
            }])
            .build();
        target.upgrade(VersionUpgradePolicy::UpgradeToV4);
        assert!(target.mappings.is_empty());
        assert_eq!(
            target.intervals,
            [SymbolInterval {
                start_ts: 1701388800000000000,
                end_ts: 1701475200000000000,
                instrument_id: u64::MAX,
                stype_in_symbol: "ES.c.0".to_owned(),
                stype_out_symbol: "ESZ3".to_owned(),
            }]
        );
    }
}
