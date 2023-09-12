//! Contains [`Metadata`] struct which comes at the beginning of any DBN file or
//! stream and [`MetadataBuilder`] for creating a [`Metadata`] with defaults.
use std::collections::HashMap;
use std::num::NonZeroU64;

// Dummy derive macro to get around `cfg_attr` incompatibility of several
// of pyo3's attribute macros. See https://github.com/PyO3/pyo3/issues/780
#[cfg(not(feature = "python"))]
pub use dbn_macros::MockPyo3;
#[cfg(feature = "serde")]
use serde::Deserialize;
use time::{macros::time, PrimitiveDateTime};

use crate::enums::{SType, Schema};
use crate::record::as_u8_slice;

/// Information about the data contained in a DBN file or stream. DBN requires the
/// Metadata to be included at the start of the encoded data.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "python", pyo3::pyclass(module = "databento_dbn"))]
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
    /// time range of the request. Otherwise, [`Self::symbol_map()`] is recommmended.
    ///
    /// # Errors
    /// This function returns an error if it can't parse a symbol into a `u32`
    /// instrument ID.
    pub fn symbol_map_for_date(&self, date: time::Date) -> crate::Result<HashMap<u32, String>> {
        let datetime = PrimitiveDateTime::new(date, time!(0:00)).assume_utc();
        // need to compare with `end` as a datetime to handle midnight case
        if date < self.start().date() || self.end().map_or(false, |end| datetime >= end) {
            return Err(crate::Error::BadArgument {
                param_name: "date".to_owned(),
                desc: "Outside the query range".to_owned(),
            });
        }
        let mut index = HashMap::new();
        for mapping in self.mappings.iter() {
            if let Some(interval) = mapping
                .intervals
                .iter()
                .find(|interval| date >= interval.start_date && date < interval.end_date)
            {
                // handle old symbology format
                if interval.symbol.is_empty() {
                    continue;
                }
                let iid = interval
                    .symbol
                    .parse()
                    .map_err(|_| crate::Error::conversion::<u32>(interval.symbol.as_str()))?;
                index.insert(iid, mapping.raw_symbol.clone());
            }
        }
        Ok(index)
    }

    /// Creates a symbology mapping from instrument ID and date to text symbol.
    ///
    /// If you're working with a single date or otherwise don't expect the mappings to
    /// change, [`Self::symbol_map_for_date()`] is recommended.
    ///
    /// # Errors
    /// This function returns an error if it can't parse a symbol into a `u32`
    /// instrument ID.
    pub fn symbol_map(&self) -> crate::Result<HashMap<(time::Date, u32), String>> {
        let mut index = HashMap::new();
        for mapping in self.mappings.iter() {
            for interval in mapping.intervals.iter() {
                // handle old symbology format
                if interval.symbol.is_empty() {
                    continue;
                }
                let mut day = interval.start_date;
                let iid = interval
                    .symbol
                    .parse()
                    .map_err(|_| crate::Error::conversion::<u32>(interval.symbol.as_str()))?;
                loop {
                    index.insert((day, iid), mapping.raw_symbol.clone());
                    day = day.next_day().unwrap();
                    if day == interval.end_date {
                        break;
                    }
                }
            }
        }
        Ok(index)
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
    /// The symbol assigned by publisher.
    pub raw_symbol: String,
    /// The mappings of `native` for different date ranges.
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
    /// The resolved symbol for this interval.
    #[cfg_attr(feature = "serde", serde(rename = "s"))]
    pub symbol: String,
}

/// The date format used for date strings when serializing [`Metadata`].
pub const DATE_FORMAT: &[time::format_description::FormatItem<'static>] =
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
    use time::macros::{date, datetime};

    use crate::publishers::Dataset;

    use super::*;

    fn metadata_w_mappings() -> Metadata {
        MetadataBuilder::new()
            .dataset(Dataset::XnasItch.as_str().to_owned())
            .schema(Some(Schema::Trades))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .start(datetime!(2023-07-01 00:00 UTC).unix_timestamp_nanos() as u64)
            .end(NonZeroU64::new(
                datetime!(2023-08-01 00:00 UTC).unix_timestamp_nanos() as u64,
            ))
            .mappings(vec![
                SymbolMapping {
                    raw_symbol: "AAPL".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date: date!(2023 - 07 - 01),
                        end_date: date!(2023 - 08 - 01),
                        symbol: "32".to_owned(),
                    }],
                },
                SymbolMapping {
                    raw_symbol: "TSLA".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2023 - 07 - 01),
                            end_date: date!(2023 - 07 - 03),
                            symbol: "10221".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 03),
                            end_date: date!(2023 - 07 - 05),
                            symbol: "10213".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 05),
                            end_date: date!(2023 - 07 - 06),
                            symbol: "10209".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 06),
                            end_date: date!(2023 - 07 - 07),
                            symbol: "10206".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 07),
                            end_date: date!(2023 - 07 - 10),
                            symbol: "10201".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 10),
                            end_date: date!(2023 - 07 - 11),
                            symbol: "10193".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 11),
                            end_date: date!(2023 - 07 - 12),
                            symbol: "10192".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 12),
                            end_date: date!(2023 - 07 - 13),
                            symbol: "10189".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 13),
                            end_date: date!(2023 - 07 - 14),
                            symbol: "10191".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 14),
                            end_date: date!(2023 - 07 - 17),
                            symbol: "10188".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 17),
                            end_date: date!(2023 - 07 - 20),
                            symbol: "10186".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 20),
                            end_date: date!(2023 - 07 - 21),
                            symbol: "10184".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 21),
                            end_date: date!(2023 - 07 - 24),
                            symbol: "10181".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 24),
                            end_date: date!(2023 - 07 - 25),
                            symbol: "10174".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 25),
                            end_date: date!(2023 - 07 - 26),
                            symbol: "10172".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 26),
                            end_date: date!(2023 - 07 - 27),
                            symbol: "10169".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 27),
                            end_date: date!(2023 - 07 - 28),
                            symbol: "10168".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 28),
                            end_date: date!(2023 - 07 - 31),
                            symbol: "10164".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 31),
                            end_date: date!(2023 - 08 - 01),
                            symbol: "10163".to_owned(),
                        },
                    ],
                },
                SymbolMapping {
                    raw_symbol: "MSFT".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2023 - 07 - 01),
                            end_date: date!(2023 - 07 - 03),
                            symbol: "6854".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 03),
                            end_date: date!(2023 - 07 - 05),
                            symbol: "6849".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 05),
                            end_date: date!(2023 - 07 - 06),
                            symbol: "6846".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 06),
                            end_date: date!(2023 - 07 - 07),
                            symbol: "6843".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 07),
                            end_date: date!(2023 - 07 - 10),
                            symbol: "6840".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 10),
                            end_date: date!(2023 - 07 - 11),
                            symbol: "6833".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 11),
                            end_date: date!(2023 - 07 - 12),
                            symbol: "6830".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 12),
                            end_date: date!(2023 - 07 - 13),
                            symbol: "6826".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 13),
                            end_date: date!(2023 - 07 - 17),
                            symbol: "6827".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 17),
                            end_date: date!(2023 - 07 - 18),
                            symbol: "6824".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 18),
                            end_date: date!(2023 - 07 - 19),
                            symbol: "6823".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 19),
                            end_date: date!(2023 - 07 - 20),
                            symbol: "6822".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 20),
                            end_date: date!(2023 - 07 - 21),
                            symbol: "6818".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 21),
                            end_date: date!(2023 - 07 - 24),
                            symbol: "6815".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 24),
                            end_date: date!(2023 - 07 - 25),
                            symbol: "6814".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 25),
                            end_date: date!(2023 - 07 - 26),
                            symbol: "6812".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 26),
                            end_date: date!(2023 - 07 - 27),
                            symbol: "6810".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 27),
                            end_date: date!(2023 - 07 - 28),
                            symbol: "6808".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 28),
                            end_date: date!(2023 - 07 - 31),
                            symbol: "6805".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 31),
                            end_date: date!(2023 - 08 - 01),
                            symbol: "6803".to_owned(),
                        },
                    ],
                },
                SymbolMapping {
                    raw_symbol: "NVDA".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2023 - 07 - 01),
                            end_date: date!(2023 - 07 - 03),
                            symbol: "7348".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 03),
                            end_date: date!(2023 - 07 - 05),
                            symbol: "7343".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 05),
                            end_date: date!(2023 - 07 - 06),
                            symbol: "7340".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 06),
                            end_date: date!(2023 - 07 - 07),
                            symbol: "7337".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 07),
                            end_date: date!(2023 - 07 - 10),
                            symbol: "7335".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 10),
                            end_date: date!(2023 - 07 - 11),
                            symbol: "7328".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 11),
                            end_date: date!(2023 - 07 - 12),
                            symbol: "7325".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 12),
                            end_date: date!(2023 - 07 - 13),
                            symbol: "7321".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 13),
                            end_date: date!(2023 - 07 - 17),
                            symbol: "7322".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 17),
                            end_date: date!(2023 - 07 - 18),
                            symbol: "7320".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 18),
                            end_date: date!(2023 - 07 - 19),
                            symbol: "7319".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 19),
                            end_date: date!(2023 - 07 - 20),
                            symbol: "7318".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 20),
                            end_date: date!(2023 - 07 - 21),
                            symbol: "7314".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 21),
                            end_date: date!(2023 - 07 - 24),
                            symbol: "7311".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 24),
                            end_date: date!(2023 - 07 - 25),
                            symbol: "7310".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 25),
                            end_date: date!(2023 - 07 - 26),
                            symbol: "7308".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 26),
                            end_date: date!(2023 - 07 - 27),
                            symbol: "7303".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 27),
                            end_date: date!(2023 - 07 - 28),
                            symbol: "7301".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 28),
                            end_date: date!(2023 - 07 - 31),
                            symbol: "7298".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 31),
                            end_date: date!(2023 - 08 - 01),
                            symbol: "7295".to_owned(),
                        },
                    ],
                },
                SymbolMapping {
                    raw_symbol: "PLTR".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2023 - 07 - 01),
                            end_date: date!(2023 - 07 - 03),
                            symbol: "8043".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 03),
                            end_date: date!(2023 - 07 - 05),
                            symbol: "8038".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 05),
                            end_date: date!(2023 - 07 - 06),
                            symbol: "8035".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 06),
                            end_date: date!(2023 - 07 - 07),
                            symbol: "8032".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 07),
                            end_date: date!(2023 - 07 - 10),
                            symbol: "8029".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 10),
                            end_date: date!(2023 - 07 - 11),
                            symbol: "8022".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 11),
                            end_date: date!(2023 - 07 - 12),
                            symbol: "8019".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 12),
                            end_date: date!(2023 - 07 - 13),
                            symbol: "8015".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 13),
                            end_date: date!(2023 - 07 - 17),
                            symbol: "8016".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 17),
                            end_date: date!(2023 - 07 - 19),
                            symbol: "8014".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 19),
                            end_date: date!(2023 - 07 - 20),
                            symbol: "8013".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 20),
                            end_date: date!(2023 - 07 - 21),
                            symbol: "8009".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 21),
                            end_date: date!(2023 - 07 - 24),
                            symbol: "8006".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 24),
                            end_date: date!(2023 - 07 - 25),
                            symbol: "8005".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 25),
                            end_date: date!(2023 - 07 - 26),
                            symbol: "8003".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 26),
                            end_date: date!(2023 - 07 - 27),
                            symbol: "7999".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 27),
                            end_date: date!(2023 - 07 - 28),
                            symbol: "7997".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 28),
                            end_date: date!(2023 - 07 - 31),
                            symbol: "7994".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2023 - 07 - 31),
                            end_date: date!(2023 - 08 - 01),
                            // test old symbology format
                            symbol: String::new(),
                        },
                    ],
                },
            ])
            .build()
    }

    #[test]
    fn test_symbol_map_for_date() {
        let target = metadata_w_mappings();
        let symbol_map_for_date = target.symbol_map_for_date(date!(2023 - 07 - 31)).unwrap();
        assert_eq!(symbol_map_for_date.len(), 4);
        assert_eq!(symbol_map_for_date[&32], "AAPL");
        assert_eq!(symbol_map_for_date[&7295], "NVDA");
        // NVDA from previous day
        assert!(!symbol_map_for_date.contains_key(&7298));
        assert_eq!(symbol_map_for_date[&10163], "TSLA");
        assert_eq!(symbol_map_for_date[&6803], "MSFT");
    }

    #[test]
    fn test_symbol_map_for_date_out_of_range() {
        let mut target = metadata_w_mappings();
        let mut res = target.symbol_map_for_date(date!(2023 - 08 - 01));
        assert!(
            matches!(res, Err(crate::Error::BadArgument { param_name, desc: _ }) if param_name == "date")
        );
        res = target.symbol_map_for_date(date!(2023 - 06 - 30));
        assert!(
            matches!(res, Err(crate::Error::BadArgument { param_name, desc: _ }) if param_name == "date")
        );
        target.end = NonZeroU64::new(datetime!(2023-07-01 08:00 UTC).unix_timestamp_nanos() as u64);
        assert!(target.symbol_map_for_date(date!(2023 - 07 - 01)).is_ok());
        assert!(target.symbol_map_for_date(date!(2023 - 07 - 02)).is_err());
        target.end = NonZeroU64::new(datetime!(2023-07-02 00:00 UTC).unix_timestamp_nanos() as u64);
        assert!(target.symbol_map_for_date(date!(2023 - 07 - 02)).is_err());
        target.end = NonZeroU64::new(
            datetime!(2023-07-02 00:00:00.000000001 UTC).unix_timestamp_nanos() as u64,
        );
        assert!(target.symbol_map_for_date(date!(2023 - 07 - 02)).is_ok());
    }

    #[test]
    fn test_symbol_map() {
        let target = metadata_w_mappings();
        let symbol_map = target.symbol_map().unwrap();
        assert_eq!(symbol_map[&(date!(2023 - 07 - 02), 32)], "AAPL");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 30), 32)], "AAPL");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 31), 32)], "AAPL");
        assert!(!symbol_map.contains_key(&(date!(2023 - 08 - 01), 32)));
        assert_eq!(symbol_map[&(date!(2023 - 07 - 08), 8029)], "PLTR");
        assert!(!symbol_map.contains_key(&(date!(2023 - 07 - 10), 8029)));
        assert_eq!(symbol_map[&(date!(2023 - 07 - 10), 8022)], "PLTR");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 20), 10184)], "TSLA");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 21), 10181)], "TSLA");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 24), 10174)], "TSLA");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 25), 10172)], "TSLA");
    }
}
