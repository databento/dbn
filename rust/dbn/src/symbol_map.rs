//! Maps for mapping instrument IDs to human-readable symbols.

use std::{cmp::Ordering, collections::HashMap, ops::Deref, sync::Arc};

use time::{macros::time, PrimitiveDateTime};

use crate::{
    compat, v1, Error, HasRType, Metadata, RType, Record, RecordRef, SType, SymbolMappingMsg,
};

/// A timeseries symbol map. Generally useful for working with historical data
/// and is commonly built from a [`Metadata`] object via [`Self::from_metadata()`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TsSymbolMap(HashMap<(time::Date, u32), Arc<String>>);

/// A point-in-time symbol map. Useful for working with live symbology or a
/// historical request over a single day or other situations where the symbol
/// mappings are known not to change.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PitSymbolMap(HashMap<u32, String>);

/// Used for retrieving a symbol mapping for a DBN record.
pub trait SymbolIndex {
    /// Returns the associated symbol mapping for `record`. Returns `None` if no mapping
    /// exists.
    fn get_for_rec<R: Record>(&self, record: &R) -> Option<&String>;

    /// Returns the associated symbol mapping for `rec_ref`. Returns `None` if no mapping
    /// exists.
    #[deprecated(
        since = "0.13.0",
        note = "The trait bound for get_for_rec was loosened to accept RecordRefs, making this function redundant"
    )]
    fn get_for_rec_ref(&self, rec_ref: RecordRef) -> Option<&String> {
        self.get_for_rec(&rec_ref)
    }
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

    /// Returns the number of symbol mappings in the map.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Creates a new timeseries symbol map from the metadata.
    ///
    /// # Errors
    /// This function returns an error if neither stype_in or stype_out are
    /// [`SType::InstrumentId`]. It will also return an error if it can't
    /// parse a symbol into `u32` instrument ID.
    pub fn from_metadata(metadata: &Metadata) -> crate::Result<Self> {
        Self::try_from(metadata)
    }

    /// Inserts a new mapping into the symbol map.
    ///
    /// If the map already had a mapping, the mapping is updated.
    ///
    /// # Errors
    /// This function returns an error if `start_date` comes after `end_date`.
    pub fn insert(
        &mut self,
        instrument_id: u32,
        start_date: time::Date,
        end_date: time::Date,
        symbol: Arc<String>,
    ) -> crate::Result<()> {
        match start_date.cmp(&end_date) {
            Ordering::Less => {
                let mut day = start_date;
                loop {
                    self.0.insert((day, instrument_id), symbol.clone());
                    day = day.next_day().unwrap();
                    if day >= end_date {
                        break;
                    }
                }
                Ok(())
            }
            Ordering::Equal => {
                // Shouldn't happen but better to just ignore
                Ok(())
            }
            Ordering::Greater => Err(Error::BadArgument {
                param_name: "start_date".to_owned(),
                desc: "start_date cannot come after end_date".to_owned(),
            }),
        }
    }

    /// Returns the symbol mapping for the given date and instrument ID. Returns `None`
    /// if no mapping exists.
    pub fn get(&self, date: time::Date, instrument_id: u32) -> Option<&String> {
        self.0.get(&(date, instrument_id)).map(Deref::deref)
    }

    /// Returns a reference to the inner map.
    pub fn inner(&self) -> &HashMap<(time::Date, u32), Arc<String>> {
        &self.0
    }

    /// Returns a mutable reference to the inner map.
    pub fn inner_mut(&mut self) -> &mut HashMap<(time::Date, u32), Arc<String>> {
        &mut self.0
    }
}

impl SymbolIndex for TsSymbolMap {
    fn get_for_rec<R: Record>(&self, record: &R) -> Option<&String> {
        record
            .index_date()
            .and_then(|date| self.get(date, record.header().instrument_id))
    }
}

impl TryFrom<&Metadata> for TsSymbolMap {
    type Error = Error;

    fn try_from(metadata: &Metadata) -> Result<Self, Error> {
        let mut res = Self::new();
        if is_inverse(metadata)? {
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
                    let symbol = Arc::new(interval.symbol.clone());
                    res.insert(iid, interval.start_date, interval.end_date, symbol)?;
                }
            }
        } else {
            for mapping in metadata.mappings.iter() {
                let symbol = Arc::new(mapping.raw_symbol.clone());
                for interval in mapping.intervals.iter() {
                    // handle old symbology format
                    if interval.symbol.is_empty() {
                        continue;
                    }
                    let iid = interval
                        .symbol
                        .parse()
                        .map_err(|_| crate::Error::conversion::<u32>(interval.symbol.clone()))?;
                    res.insert(iid, interval.start_date, interval.end_date, symbol.clone())?;
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
    /// [`SType::InstrumentId`]. It will also return an error if it can't
    /// parse a symbol into `u32` instrument ID or if `date` is outside the query range
    /// of the metadata.
    pub fn from_metadata(metadata: &Metadata, date: time::Date) -> crate::Result<Self> {
        let is_inverse = is_inverse(metadata)?;
        let datetime = PrimitiveDateTime::new(date, time!(0:00)).assume_utc();
        // need to compare with `end` as a datetime to handle midnight case
        if date < metadata.start().date() || metadata.end().map_or(false, |end| datetime >= end) {
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
    /// This function returns an error when `record` contains a [`SymbolMappingMsg`] but
    /// it contains invalid UTF-8.
    pub fn on_record(&mut self, record: RecordRef) -> crate::Result<()> {
        if matches!(record.rtype(), Ok(RType::SymbolMapping)) {
            // >= to allow WithTsOut
            if record.record_size() >= std::mem::size_of::<SymbolMappingMsg>() {
                // Safety: checked rtype and length
                self.on_symbol_mapping(unsafe { record.get_unchecked::<SymbolMappingMsg>() })
            } else {
                // Use get here to get still perform length checks
                self.on_symbol_mapping(record.get::<v1::SymbolMappingMsg>().unwrap())
            }
        } else {
            Ok(())
        }
    }

    /// Handles updating the mappings for a symbol mapping record.
    ///
    /// # Errors
    /// This function returns an error when `symbol_mapping` contains invalid UTF-8.
    pub fn on_symbol_mapping<S: compat::SymbolMappingRec>(
        &mut self,
        symbol_mapping: &S,
    ) -> crate::Result<()> {
        let stype_out_symbol = symbol_mapping.stype_out_symbol()?;
        self.0.insert(
            symbol_mapping.header().instrument_id,
            stype_out_symbol.to_owned(),
        );
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
        self.get(record.header().instrument_id)
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
        self.get(index.0, index.1)
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

fn is_inverse(metadata: &Metadata) -> crate::Result<bool> {
    match (metadata.stype_in, metadata.stype_out) {
        (_, SType::InstrumentId) => Ok(false),
        (Some(SType::InstrumentId), _) => Ok(true),
        _ => {
            Err(Error::BadArgument {
                param_name: "metadata".to_owned(),
                desc: "Can only create symbol maps from metadata where either stype_out or stype_in is instrument ID".to_owned(),
            })
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::num::NonZeroU64;

    use rstest::rstest;
    use time::macros::{date, datetime};

    use crate::{
        compat::{SymbolMappingMsgV1, SymbolMappingRec},
        publishers::Dataset,
        MappingInterval, Metadata, Schema, SymbolMapping, UNDEF_TIMESTAMP,
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
        assert!(!symbol_map.0.contains_key(&(date!(2023 - 08 - 01), 32)));
        assert_eq!(symbol_map[&(date!(2023 - 07 - 08), 8029)], "PLTR");
        assert!(!symbol_map.0.contains_key(&(date!(2023 - 07 - 10), 8029)));
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

    // start_date == end_date is generally invalid and
    // previously caused a panic
    #[test]
    fn test_insert_start_end_date_same() {
        let mut target = TsSymbolMap::new();
        target
            .insert(
                1,
                date!(2023 - 12 - 03),
                date!(2023 - 12 - 03),
                Arc::new("test".to_owned()),
            )
            .unwrap();
        // should have no effect
        assert!(target.is_empty());
    }
}
