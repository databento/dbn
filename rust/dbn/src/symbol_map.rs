//! Maps for mapping instrument IDs to human-readable symbols.

use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt, mem,
    sync::atomic::{self, AtomicUsize},
};

use time::{macros::time, PrimitiveDateTime};

use crate::{compat, v1, Error, HasRType, Metadata, Record, RecordRef, SymbolMappingMsg};

/// A timeseries symbol map. Useful for working with historical requests over multiple
/// days, where the same instrument ID can map to different symbols at different times.
///
/// Commonly built with [`Metadata::symbol_map()`]. For live data, call
/// [`on_record()`](Self::on_record) with each incoming record to keep the map updated as
/// symbol mappings arrive.
///
/// # Examples
/// ```no_run
/// use dbn::{
///     decode::{DbnDecoder, DecodeRecord, DbnMetadata},
///     pretty, symbol_map::SymbolIndex,
///     TradeMsg,
/// };
///
/// let mut decoder = DbnDecoder::from_zstd_file("20241007.trades.dbn.zst")?;
/// let symbol_map = decoder.metadata().symbol_map()?;
///
/// while let Some(trade) = decoder.decode_record::<TradeMsg>()? {
///     if let Some(symbol) = symbol_map.get_for_rec(trade) {
///         println!("{symbol}: price={}", pretty::Px(trade.price));
///     } else {
///         eprintln!("Missing symbol for {trade:?}")
///     }
/// }
/// # Ok::<(), dbn::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TsSymbolMap(HashMap<u32, SymbolIntervals>);

/// An instrument ID's intervals within a [`TsSymbolMap`], sorted by start.
#[derive(Default)]
pub struct SymbolIntervals {
    intervals: Vec<SymbolInterval>,
    // Cached index of the last lookup.
    hint: AtomicUsize,
}

/// A symbol and the time range it applies to within a [`TsSymbolMap`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolInterval {
    /// The start of the interval (inclusive) expressed as the number of nanoseconds
    /// since the UNIX epoch.
    pub start_ts: u64,
    /// The end of the interval (exclusive) expressed as the number of nanoseconds since
    /// the UNIX epoch.
    pub end_ts: u64,
    /// The symbol for the instrument ID during this interval.
    pub symbol: String,
}

/// A point-in-time symbol map. Useful for working with live symbology or a
/// historical request over a single day or other situations where the symbol
/// mappings are known not to change.
///
/// Commonly built with [`Metadata::symbol_map_for_date()`]. For live data, call
/// [`on_record()`](Self::on_record) with each incoming record to keep the map
/// updated as symbol mappings arrive.
///
/// # Examples
///
/// Single-day historical usage:
/// ```no_run
/// use dbn::{
///     decode::{DbnDecoder, DecodeRecord, DbnMetadata},
///     pretty,
///     symbol_map::SymbolIndex,
///     Mbp1Msg,
/// };
///
/// let mut decoder = DbnDecoder::from_zstd_file("mbp1.dbn.zst")?;
/// let date = decoder.metadata().start().date();
/// let symbol_map = decoder.metadata().symbol_map_for_date(date)?;
///
/// while let Some(mbp) = decoder.decode_record::<Mbp1Msg>()? {
///     if let Some(symbol) = symbol_map.get_for_rec(mbp) {
///         println!(
///             "{symbol}: bid={} ask={}",
///             pretty::Px(mbp.levels[0].bid_px),
///             pretty::Px(mbp.levels[0].ask_px)
///         );
///     } else {
///         eprintln!("Missing symbol for {mbp:?}")
///     }
/// }
/// # Ok::<(), dbn::Error>(())
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PitSymbolMap(HashMap<u32, String>);

/// Retrieves a symbol mapping for a DBN record. Implemented by both
/// [`TsSymbolMap`] and [`PitSymbolMap`], allowing code to be generic over the
/// type of symbol map used.
pub trait SymbolIndex {
    /// Returns the associated symbol mapping for `record`. Returns `None` if no mapping
    /// exists.
    fn get_for_rec<R: Record>(&self, record: &R) -> Option<&String>;
}

impl TsSymbolMap {
    /// Creates a new timeseries symbol map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if there are no mappings.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of intervals in the map.
    pub fn len(&self) -> usize {
        self.0
            .values()
            .map(|intervals| intervals.intervals.len())
            .sum()
    }

    /// Creates a new timeseries symbol map from the metadata.
    ///
    /// # Errors
    /// This function returns an error if neither stype_in or stype_out are
    /// [`SType::InstrumentId`](crate::SType::InstrumentId). It will also return an
    /// error if it can't parse a symbol into `u32` instrument ID.
    pub fn from_metadata(metadata: &Metadata) -> crate::Result<Self> {
        Self::try_from(metadata)
    }

    /// Inserts a new mapping into the symbol map from `start_ts` (inclusive) to `end_ts`
    /// (exclusive), both expressed as the number of nanoseconds since the UNIX epoch.
    ///
    /// Where the map already had a mapping for `instrument_id` between `start_ts` and
    /// `end_ts`, the new mapping replaces it.
    ///
    /// # Errors
    /// This function returns an error if `start_ts` comes after `end_ts`.
    pub fn insert(
        &mut self,
        instrument_id: u32,
        start_ts: u64,
        end_ts: u64,
        symbol: impl Into<String>,
    ) -> crate::Result<()> {
        match start_ts.cmp(&end_ts) {
            Ordering::Less => {
                let intervals = &mut self.0.entry(instrument_id).or_default().intervals;
                let new = SymbolInterval {
                    start_ts,
                    end_ts,
                    symbol: symbol.into(),
                };
                // Mappings usually arrive in time order, so most inserts append
                if intervals.last().is_none_or(|last| last.end_ts <= start_ts) {
                    match intervals.last_mut() {
                        Some(last) if last.end_ts == start_ts && last.symbol == new.symbol => {
                            last.end_ts = end_ts;
                        }
                        _ => intervals.push(new),
                    }
                } else {
                    let old = mem::take(intervals);
                    let before =
                        old.iter()
                            .filter(|i| i.start_ts < start_ts)
                            .map(|i| SymbolInterval {
                                end_ts: i.end_ts.min(start_ts),
                                ..i.clone()
                            });
                    let after = old
                        .iter()
                        .filter(|i| i.end_ts > end_ts)
                        .map(|i| SymbolInterval {
                            start_ts: i.start_ts.max(end_ts),
                            ..i.clone()
                        });
                    *intervals = before.chain([new]).chain(after).collect();
                    intervals.dedup_by(|next, prev| {
                        let adjacent = prev.end_ts == next.start_ts && prev.symbol == next.symbol;
                        if adjacent {
                            prev.end_ts = next.end_ts;
                        }
                        adjacent
                    });
                }
                Ok(())
            }
            Ordering::Equal => {
                // Shouldn't happen but better to just ignore
                Ok(())
            }
            Ordering::Greater => Err(Error::BadArgument {
                param_name: "start_ts".to_owned(),
                desc: "start_ts cannot come after end_ts".to_owned(),
            }),
        }
    }

    /// Returns the symbol mapping for the given timestamp, expressed as the number of
    /// nanoseconds since the UNIX epoch, and instrument ID. Returns `None` if no mapping
    /// exists.
    pub fn get_for_ts(&self, ts: u64, instrument_id: u32) -> Option<&String> {
        let SymbolIntervals { intervals, hint } = self.0.get(&instrument_id)?;
        // helper to check the interval at index `i` for `ts`
        let includes_ts = |i: usize| {
            intervals
                .get(i)
                .is_some_and(|interval| interval.start_ts <= ts && ts < interval.end_ts)
        };
        let hint_idx = hint.load(atomic::Ordering::Relaxed);
        let idx = if includes_ts(hint_idx) {
            hint_idx
        } else {
            let idx = if includes_ts(hint_idx + 1) {
                hint_idx + 1
            } else {
                intervals
                    .partition_point(|i| i.start_ts <= ts)
                    .checked_sub(1)?
            };
            hint.store(idx, atomic::Ordering::Relaxed);
            idx
        };
        let interval = &intervals[idx];
        (ts < interval.end_ts).then_some(&interval.symbol)
    }

    /// Returns the symbol mapping for the start of the given UTC date and instrument ID.
    /// Returns `None` if no mapping exists.
    #[deprecated(since = "0.72.0", note = "Use `get_for_ts()` instead")]
    pub fn get(&self, date: time::Date, instrument_id: u32) -> Option<&String> {
        self.get_for_ts(date_to_ts(date), instrument_id)
    }

    /// Handles updating the mappings (if required) for a generic record.
    ///
    /// # Errors
    /// This function returns an error when `record` contains a symbol mapping
    /// with invalid UTF-8.
    pub fn on_record(&mut self, record: RecordRef) -> crate::Result<()> {
        if let Ok(symbol_mapping) = record.try_get::<SymbolMappingMsg>() {
            self.on_symbol_mapping(symbol_mapping)
        } else if let Ok(symbol_mapping) = record.try_get::<v1::SymbolMappingMsg>() {
            self.on_symbol_mapping(symbol_mapping)
        } else {
            Ok(())
        }
    }

    /// Handles updating the mappings from a symbol mapping record. A mapping without a
    /// start applies from the UNIX epoch and one without an end applies indefinitely.
    ///
    /// # Errors
    /// This function returns an error if the symbol contains invalid UTF-8 or the
    /// mapping starts after it ends.
    pub fn on_symbol_mapping<S: compat::SymbolMappingRec>(
        &mut self,
        symbol_mapping: &S,
    ) -> crate::Result<()> {
        let start_ts = symbol_mapping
            .start_ts()
            .map(|start| start.unix_timestamp_nanos() as u64)
            .unwrap_or(0);
        let end_ts = symbol_mapping
            .end_ts()
            .map(|end| end.unix_timestamp_nanos() as u64)
            .unwrap_or(u64::MAX);
        self.insert(
            symbol_mapping.instrument_id() as u32,
            start_ts,
            end_ts,
            symbol_mapping.stype_out_symbol()?,
        )
    }

    /// Returns a reference to the inner map of each instrument ID's intervals.
    pub fn inner(&self) -> &HashMap<u32, SymbolIntervals> {
        &self.0
    }

    /// Returns a mutable reference to the inner map of each instrument ID's intervals.
    ///
    /// [`get_for_ts()`](Self::get_for_ts) and [`insert()`](Self::insert) rely on
    /// each instrument ID's intervals being non-empty, sorted by start, and
    /// non-overlapping, and
    /// equality relies on adjacent intervals having different symbols. Modifications
    /// must preserve this.
    pub fn inner_mut(&mut self) -> &mut HashMap<u32, SymbolIntervals> {
        &mut self.0
    }
}

impl SymbolIntervals {
    /// Returns the intervals, sorted by start.
    pub fn intervals(&self) -> &[SymbolInterval] {
        &self.intervals
    }

    /// Returns a mutable reference to the intervals.
    ///
    /// See [`TsSymbolMap::inner_mut()`] for the invariants modifications must preserve.
    pub fn intervals_mut(&mut self) -> &mut Vec<SymbolInterval> {
        &mut self.intervals
    }
}

impl Clone for SymbolIntervals {
    fn clone(&self) -> Self {
        Self {
            intervals: self.intervals.clone(),
            hint: AtomicUsize::new(self.hint.load(atomic::Ordering::Relaxed)),
        }
    }
}

impl PartialEq for SymbolIntervals {
    fn eq(&self, other: &Self) -> bool {
        self.intervals == other.intervals
    }
}

impl Eq for SymbolIntervals {}

impl fmt::Debug for SymbolIntervals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.intervals.fmt(f)
    }
}

impl SymbolIndex for TsSymbolMap {
    fn get_for_rec<R: Record>(&self, record: &R) -> Option<&String> {
        // A null timestamp is `u64::MAX`, which no half-open interval contains
        self.get_for_ts(record.raw_index_ts(), record.instrument_id() as u32)
    }
}

impl TryFrom<&Metadata> for TsSymbolMap {
    type Error = Error;

    fn try_from(metadata: &Metadata) -> Result<Self, Error> {
        let mut res = Self::new();
        if metadata.is_inverse()? {
            for mapping in metadata.mappings.iter() {
                let iid = mapping
                    .raw_symbol
                    .parse()
                    .map_err(|_| crate::Error::conversion::<u32>(mapping.raw_symbol.clone()))?;
                for interval in mapping.intervals.iter() {
                    // handle old symbology format
                    if interval.symbol.is_empty() {
                        continue;
                    }
                    res.insert(
                        iid,
                        date_to_ts(interval.start_date),
                        date_to_ts(interval.end_date),
                        interval.symbol.clone(),
                    )?;
                }
            }
        } else {
            for mapping in metadata.mappings.iter() {
                for interval in mapping.intervals.iter() {
                    // handle old symbology format
                    if interval.symbol.is_empty() {
                        continue;
                    }
                    let iid = interval
                        .symbol
                        .parse()
                        .map_err(|_| crate::Error::conversion::<u32>(interval.symbol.clone()))?;
                    res.insert(
                        iid,
                        date_to_ts(interval.start_date),
                        date_to_ts(interval.end_date),
                        mapping.raw_symbol.clone(),
                    )?;
                }
            }
        }
        Ok(res)
    }
}

impl PitSymbolMap {
    /// Creates a new empty `PitSymbolMap`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if there are no mappings.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of symbol mappings in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Creates a new `PitSymbolMap` populated with the mappings from `metadata` for
    /// `date`.
    ///
    /// # Errors
    /// This function returns an error if neither stype_in or stype_out are
    /// [`SType::InstrumentId`](crate::SType::InstrumentId). It will also return an
    /// error if it can't parse a symbol into `u32` instrument ID or if `date` is
    /// outside the query range of the metadata.
    pub fn from_metadata(metadata: &Metadata, date: time::Date) -> crate::Result<Self> {
        let is_inverse = metadata.is_inverse()?;
        let datetime = PrimitiveDateTime::new(date, time!(0:00)).assume_utc();
        // need to compare with `end` as a datetime to handle midnight case
        if date < metadata.start().date() || metadata.end().is_some_and(|end| datetime >= end) {
            return Err(crate::Error::BadArgument {
                param_name: "date".to_owned(),
                desc: "Outside the query range".to_owned(),
            });
        }
        let mut res = HashMap::new();
        for mapping in metadata.mappings.iter() {
            if let Some(interval) = mapping
                .intervals
                .iter()
                .find(|interval| date >= interval.start_date && date < interval.end_date)
            {
                // handle old symbology format
                if interval.symbol.is_empty() {
                    continue;
                }
                if is_inverse {
                    let iid = mapping.raw_symbol.parse().map_err(|_| {
                        crate::Error::conversion::<u32>(mapping.raw_symbol.as_str())
                    })?;
                    res.insert(iid, interval.symbol.clone());
                } else {
                    let iid = interval
                        .symbol
                        .parse()
                        .map_err(|_| crate::Error::conversion::<u32>(interval.symbol.as_str()))?;
                    res.insert(iid, mapping.raw_symbol.clone());
                }
            }
        }
        Ok(Self(res))
    }

    /// Handles updating the mappings (if required) for a generic record.
    ///
    /// # Errors
    /// This function returns an error when `record` contains a symbol mapping
    /// with invalid UTF-8.
    pub fn on_record(&mut self, record: RecordRef) -> crate::Result<()> {
        if let Ok(symbol_mapping) = record.try_get::<SymbolMappingMsg>() {
            self.on_symbol_mapping(symbol_mapping)
        } else if let Ok(symbol_mapping) = record.try_get::<v1::SymbolMappingMsg>() {
            self.on_symbol_mapping(symbol_mapping)
        } else {
            Ok(())
        }
    }

    /// Handles updating the mappings from a symbol mapping record.
    ///
    /// # Errors
    /// This function returns an error if the symbol contains invalid UTF-8.
    pub fn on_symbol_mapping<S: compat::SymbolMappingRec>(
        &mut self,
        symbol_mapping: &S,
    ) -> crate::Result<()> {
        let stype_out_symbol = symbol_mapping.stype_out_symbol()?;
        self.0.insert(
            symbol_mapping.instrument_id() as u32,
            stype_out_symbol.to_owned(),
        );
        Ok(())
    }

    /// Handles updating the mappings from an instrument definition record.
    ///
    /// # Errors
    /// This function returns an error if the symbol contains invalid UTF-8.
    pub fn on_instrument_def<D: compat::InstrumentDefRec>(
        &mut self,
        instrument_def: &D,
    ) -> crate::Result<()> {
        let raw_symbol = instrument_def.raw_symbol()?;
        self.0
            .insert(instrument_def.instrument_id() as u32, raw_symbol.to_owned());
        Ok(())
    }

    /// Returns a reference to the mapping for the given instrument ID.
    pub fn get(&self, instrument_id: u32) -> Option<&String> {
        self.0.get(&instrument_id)
    }

    /// Returns a reference to the inner map.
    pub fn inner(&self) -> &HashMap<u32, String> {
        &self.0
    }

    /// Returns a mutable reference to the inner map.
    pub fn inner_mut(&mut self) -> &mut HashMap<u32, String> {
        &mut self.0
    }
}

impl SymbolIndex for PitSymbolMap {
    fn get_for_rec<R: Record>(&self, record: &R) -> Option<&String> {
        self.get(record.instrument_id() as u32)
    }
}

impl<R: HasRType> std::ops::Index<&R> for TsSymbolMap {
    type Output = String;

    fn index(&self, index: &R) -> &Self::Output {
        self.get_for_rec(index).unwrap()
    }
}

impl std::ops::Index<&(time::Date, u32)> for TsSymbolMap {
    type Output = String;

    fn index(&self, index: &(time::Date, u32)) -> &Self::Output {
        self.get_for_ts(date_to_ts(index.0), index.1)
            .expect("symbol mapping for date and instrument ID")
    }
}

impl<R: HasRType> std::ops::Index<&R> for PitSymbolMap {
    type Output = String;

    fn index(&self, index: &R) -> &Self::Output {
        self.get_for_rec(index).unwrap()
    }
}

impl std::ops::Index<u32> for PitSymbolMap {
    type Output = String;

    fn index(&self, instrument_id: u32) -> &Self::Output {
        self.get(instrument_id)
            .expect("symbol mapping for instrument ID")
    }
}

pub(crate) fn date_to_ts(date: time::Date) -> u64 {
    date.midnight().assume_utc().unix_timestamp_nanos() as u64
}

#[cfg(test)]
pub(crate) mod tests {
    use std::num::NonZeroU64;

    use rstest::rstest;
    use time::macros::{date, datetime};

    use crate::{
        compat::{SymbolMappingMsgV1, SymbolMappingRec},
        publishers::Dataset,
        MappingInterval, Metadata, SType, Schema, SymbolMapping, UNDEF_TIMESTAMP,
    };

    use super::*;

    pub fn metadata_w_mappings() -> Metadata {
        Metadata::builder()
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

    fn metadata_w_inverse_mappings() -> Metadata {
        let mut metadata = metadata_w_mappings();
        metadata.stype_in = Some(SType::InstrumentId);
        metadata.stype_out = SType::RawSymbol;
        let mut new_mappings = Vec::new();
        for mapping in metadata.mappings.iter() {
            for interval in mapping.intervals.iter() {
                if interval.symbol.is_empty() {
                    continue;
                }
                new_mappings.push(SymbolMapping {
                    raw_symbol: interval.symbol.clone(),
                    intervals: vec![MappingInterval {
                        start_date: interval.start_date,
                        end_date: interval.end_date,
                        symbol: mapping.raw_symbol.clone(),
                    }],
                })
            }
        }
        metadata.mappings = new_mappings;
        metadata
    }

    #[test]
    fn test_symbol_map_for_date() {
        let target = metadata_w_mappings();
        let symbol_map_for_date = target.symbol_map_for_date(date!(2023 - 07 - 31)).unwrap();
        assert_eq!(symbol_map_for_date.len(), 4);
        assert_eq!(symbol_map_for_date[32], "AAPL");
        assert_eq!(symbol_map_for_date[7295], "NVDA");
        // NVDA from previous day
        assert!(!symbol_map_for_date.0.contains_key(&7298));
        assert_eq!(symbol_map_for_date[10163], "TSLA");
        assert_eq!(symbol_map_for_date[6803], "MSFT");

        let inverse_target = metadata_w_inverse_mappings();
        assert_eq!(
            symbol_map_for_date,
            inverse_target
                .symbol_map_for_date(date!(2023 - 07 - 31))
                .unwrap()
        );
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
        assert!(symbol_map
            .get_for_ts(date_to_ts(date!(2023 - 08 - 01)), 32)
            .is_none());
        assert_eq!(symbol_map[&(date!(2023 - 07 - 08), 8029)], "PLTR");
        assert!(symbol_map
            .get_for_ts(date_to_ts(date!(2023 - 07 - 10)), 8029)
            .is_none());
        assert_eq!(symbol_map[&(date!(2023 - 07 - 10), 8022)], "PLTR");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 20), 10184)], "TSLA");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 21), 10181)], "TSLA");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 24), 10174)], "TSLA");
        assert_eq!(symbol_map[&(date!(2023 - 07 - 25), 10172)], "TSLA");

        let inverse_target = metadata_w_inverse_mappings();
        assert_eq!(symbol_map, inverse_target.symbol_map().unwrap());
    }

    #[test]
    fn test_other_stype_errors() {
        let mut target = metadata_w_mappings();
        target.stype_out = SType::RawSymbol;
        assert!(target.symbol_map().is_err());
        assert!(target.symbol_map_for_date(date!(2023 - 07 - 31)).is_err());
    }

    #[rstest]
    #[case::v1(SymbolMappingMsgV1::default())]
    #[case::v2(SymbolMappingMsg::default())]
    fn test_on_record<S: SymbolMappingRec>(#[case] _sm: S) -> crate::Result<()> {
        let mut target = PitSymbolMap::new();
        target.on_record(RecordRef::from(&SymbolMappingMsg::new(
            1,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?))?;
        target.on_record(RecordRef::from(&SymbolMappingMsg::new(
            2,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "TSLA",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?))?;
        target.on_record(RecordRef::from(&SymbolMappingMsg::new(
            3,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "MSFT",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?))?;
        assert_eq!(
            *target.inner(),
            HashMap::from([
                (1, "AAPL".to_owned()),
                (2, "TSLA".to_owned()),
                (3, "MSFT".to_owned())
            ])
        );
        target.on_record(RecordRef::from(&SymbolMappingMsg::new(
            10,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?))?;
        assert_eq!(target[10], "AAPL");
        target.on_record(RecordRef::from(&SymbolMappingMsg::new(
            9,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "MSFT",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?))?;
        assert_eq!(target[9], "MSFT");

        Ok(())
    }

    #[test]
    fn test_on_symbol_mapping() -> crate::Result<()> {
        let mut target = PitSymbolMap::new();
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            1,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?)?;
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            2,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "TSLA",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?)?;
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            3,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "MSFT",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?)?;
        assert_eq!(
            *target.inner(),
            HashMap::from([
                (1, "AAPL".to_owned()),
                (2, "TSLA".to_owned()),
                (3, "MSFT".to_owned())
            ])
        );
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            10,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?)?;
        assert_eq!(target[10], "AAPL");
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            9,
            2,
            SType::InstrumentId,
            "",
            SType::RawSymbol,
            "MSFT",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?)?;
        assert_eq!(target[9], "MSFT");

        Ok(())
    }

    // start_ts == end_ts is generally invalid and
    // previously caused a panic
    #[test]
    fn test_insert_start_end_ts_same() {
        let mut target = TsSymbolMap::new();
        let ts = date_to_ts(date!(2023 - 12 - 03));
        target.insert(1, ts, ts, "test").unwrap();
        // should have no effect
        assert!(target.is_empty());
    }

    #[test]
    fn test_insert_matches_per_hour() {
        const HOUR: u64 = 3_600_000_000_000;
        let base = date_to_ts(date!(2026 - 09 - 01));
        let symbols = ["ESU6", "ESZ6", "ESH7"];
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut next = |n: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % n
        };
        let mut target = TsSymbolMap::new();
        let mut per_hour = HashMap::new();
        for _ in 0..500 {
            let iid = next(3) as u32;
            let start_hour = next(400);
            let end_hour = start_hour + 1 + next(120);
            let symbol = symbols[next(3) as usize];
            target
                .insert(
                    iid,
                    base + start_hour * HOUR,
                    base + end_hour * HOUR,
                    symbol,
                )
                .unwrap();
            for hour in start_hour..end_hour {
                per_hour.insert((hour, iid), symbol);
            }

            let mut hour_by_hour = TsSymbolMap::new();
            for iid in 0..3 {
                for hour in 0..540 {
                    let expected = per_hour.get(&(hour, iid)).copied();
                    let ts = base + hour * HOUR;
                    assert_eq!(target.get_for_ts(ts, iid).map(String::as_str), expected);
                    assert_eq!(
                        target.get_for_ts(ts + HOUR - 1, iid).map(String::as_str),
                        expected
                    );
                    if let Some(symbol) = expected {
                        hour_by_hour.insert(iid, ts, ts + HOUR, symbol).unwrap();
                    }
                }
            }
            assert_eq!(target, hour_by_hour);
        }
    }

    #[test]
    fn test_ts_on_symbol_mapping() -> crate::Result<()> {
        let remap_ts = datetime!(2026-09-16 13:30 UTC).unix_timestamp_nanos() as u64;
        let mut target = TsSymbolMap::new();
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            42140870,
            2,
            SType::Continuous,
            "ES.v.0",
            SType::RawSymbol,
            "ESU6",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?)?;
        target.on_symbol_mapping(&SymbolMappingMsg::new(
            42140870,
            2,
            SType::Continuous,
            "ES.v.0",
            SType::RawSymbol,
            "ESZ6",
            remap_ts,
            UNDEF_TIMESTAMP,
        )?)?;
        assert_eq!(target.len(), 2);
        assert_eq!(target.get_for_ts(0, 42140870).unwrap(), "ESU6");
        assert_eq!(target.get_for_ts(remap_ts - 1, 42140870).unwrap(), "ESU6");
        assert_eq!(target.get_for_ts(remap_ts, 42140870).unwrap(), "ESZ6");
        assert_eq!(target.get_for_ts(u64::MAX - 1, 42140870).unwrap(), "ESZ6");
        assert!(target.get_for_ts(UNDEF_TIMESTAMP, 42140870).is_none());
        Ok(())
    }

    #[test]
    fn test_overlapping_continuous_mappings() {
        let metadata = Metadata::builder()
            .dataset(Dataset::GlbxMdp3.as_str().to_owned())
            .schema(Some(Schema::Ohlcv1D))
            .stype_in(Some(SType::Continuous))
            .stype_out(SType::InstrumentId)
            .start(datetime!(2026-09-01 00:00 UTC).unix_timestamp_nanos() as u64)
            .end(NonZeroU64::new(
                datetime!(2026-09-26 00:00 UTC).unix_timestamp_nanos() as u64,
            ))
            .mappings(vec![
                SymbolMapping {
                    raw_symbol: "ES.c.0".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2026 - 09 - 01),
                            end_date: date!(2026 - 09 - 20),
                            symbol: "42140870".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2026 - 09 - 20),
                            end_date: date!(2026 - 09 - 26),
                            symbol: "10252".to_owned(),
                        },
                    ],
                },
                SymbolMapping {
                    raw_symbol: "ES.v.0".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: date!(2026 - 09 - 01),
                            end_date: date!(2026 - 09 - 16),
                            symbol: "42140870".to_owned(),
                        },
                        MappingInterval {
                            start_date: date!(2026 - 09 - 16),
                            end_date: date!(2026 - 09 - 26),
                            symbol: "10252".to_owned(),
                        },
                    ],
                },
            ])
            .build();
        let target = metadata.symbol_map().unwrap();
        assert_eq!(target.len(), 3);
        assert_eq!(target[&(date!(2026 - 09 - 10), 42140870)], "ES.v.0");
        assert_eq!(target[&(date!(2026 - 09 - 17), 42140870)], "ES.c.0");
        assert!(target
            .get_for_ts(date_to_ts(date!(2026 - 09 - 20)), 42140870)
            .is_none());
        assert!(target
            .get_for_ts(date_to_ts(date!(2026 - 09 - 15)), 10252)
            .is_none());
        assert_eq!(target[&(date!(2026 - 09 - 17), 10252)], "ES.v.0");
        assert_eq!(target[&(date!(2026 - 09 - 22), 10252)], "ES.v.0");
    }
}
