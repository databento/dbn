//! Splitters for routing DBN records to different encoders.
//!
//! This module provides [`SplitEncoder`] which wraps a [`Splitter`] implementation
//! to route records to different sub-encoders based on various criteria such as time,
//! symbol, or schema.

use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    num::NonZeroU64,
};

use time::{Time, Weekday};

use crate::{
    encode::{DbnEncodable, EncodeDbn, EncodeRecord, EncodeRecordRef, EncodeRecordTextExt},
    Metadata, RType, Record, RecordRef, Schema, SymbolIndex,
};

/// A strategy for routing records to different sub-encoders.
pub trait Splitter<E> {
    /// Returns the encoder for the given record, or `None` if the record should be ignored.
    ///
    /// # Errors
    /// This function returns an error if it fails to create the sub encoder.
    fn sub_encoder<R>(
        &mut self,
        metadata: Option<&Metadata>,
        record: &R,
    ) -> crate::Result<Option<&mut E>>
    where
        R: Record;

    /// Returns an iterator over all active sub-encoders.
    fn sub_encoders<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
    where
        E: 'a;
}

/// An encoder that routes records to sub-encoders based on a [`Splitter`] strategy.
///
/// Wraps a `Splitter` implementation and delegates encoding to the appropriate
/// sub-encoder returned by the splitter for each record.
#[derive(Debug)]
pub struct SplitEncoder<S, E> {
    splitter: S,
    metadata: Option<Metadata>,
    _encoder: PhantomData<E>,
}

impl<S, E> SplitEncoder<S, E> {
    /// Creates a new `SplitEncoder` without metadata.
    ///
    /// Use this when encoding records without associated metadata, such
    /// as DBN fragments.
    pub fn records_only(splitter: S) -> Self {
        Self {
            splitter,
            metadata: None,
            _encoder: PhantomData,
        }
    }

    /// Creates a new `SplitEncoder` with metadata.
    ///
    /// The metadata will be passed to the splitter and used to create split-specific
    /// metadata for each sub-encoder.
    pub fn with_metadata(splitter: S, metadata: Metadata) -> Self {
        Self {
            splitter,
            metadata: Some(metadata),
            _encoder: PhantomData,
        }
    }
}

impl<S, E: EncodeRecord> EncodeRecord for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: EncodeRecord,
{
    fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> crate::Result<()> {
        if let Some(encoder) = self.splitter.sub_encoder(self.metadata.as_ref(), record)? {
            encoder.encode_record(record)?;
        }
        Ok(())
    }

    fn flush(&mut self) -> crate::Result<()> {
        self.splitter.sub_encoders().try_for_each(E::flush)
    }
}

impl<S, E> EncodeRecordRef for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: EncodeRecordRef,
{
    fn encode_record_ref(&mut self, record: RecordRef) -> crate::Result<()> {
        if let Some(encoder) = self.splitter.sub_encoder(self.metadata.as_ref(), &record)? {
            encoder.encode_record_ref(record)?;
        }
        Ok(())
    }

    unsafe fn encode_record_ref_ts_out(
        &mut self,
        record: RecordRef,
        ts_out: bool,
    ) -> crate::Result<()> {
        if let Some(encoder) = self.splitter.sub_encoder(self.metadata.as_ref(), &record)? {
            encoder.encode_record_ref_ts_out(record, ts_out)?;
        }
        Ok(())
    }
}

impl<S, E> EncodeRecordTextExt for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: EncodeRecordTextExt,
{
    fn encode_record_with_sym<R: DbnEncodable>(
        &mut self,
        record: &R,
        symbol: Option<&str>,
    ) -> crate::Result<()> {
        if let Some(encoder) = self.splitter.sub_encoder(self.metadata.as_ref(), record)? {
            encoder.encode_record_with_sym(record, symbol)?;
        }
        Ok(())
    }
}

impl<S, E> EncodeDbn for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: EncodeRecordTextExt,
{
}

#[cfg(feature = "async")]
impl<S, E> super::AsyncEncodeRecord for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: super::AsyncEncodeRecord,
{
    async fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> crate::Result<()> {
        if let Some(encoder) = self.splitter.sub_encoder(self.metadata.as_ref(), record)? {
            encoder.encode_record(record).await?;
        }
        Ok(())
    }

    async fn flush(&mut self) -> crate::Result<()> {
        for encoder in self.splitter.sub_encoders() {
            encoder.flush().await?;
        }
        Ok(())
    }

    async fn shutdown(&mut self) -> crate::Result<()> {
        for encoder in self.splitter.sub_encoders() {
            encoder.shutdown().await?;
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<S, E> super::AsyncEncodeRecordRef for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: super::AsyncEncodeRecordRef,
{
    async fn encode_record_ref(&mut self, record_ref: RecordRef<'_>) -> crate::Result<()> {
        if let Some(encoder) = self
            .splitter
            .sub_encoder(self.metadata.as_ref(), &record_ref)?
        {
            encoder.encode_record_ref(record_ref).await?;
        }
        Ok(())
    }

    async unsafe fn encode_record_ref_ts_out(
        &mut self,
        record_ref: RecordRef<'_>,
        ts_out: bool,
    ) -> crate::Result<()> {
        if let Some(encoder) = self
            .splitter
            .sub_encoder(self.metadata.as_ref(), &record_ref)?
        {
            encoder.encode_record_ref_ts_out(record_ref, ts_out).await?;
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
impl<S, E> super::AsyncEncodeRecordTextExt for SplitEncoder<S, E>
where
    S: Splitter<E>,
    E: super::AsyncEncodeRecordTextExt,
{
    async fn encode_record_with_sym<R: DbnEncodable>(
        &mut self,
        record: &R,
        symbol: Option<&str>,
    ) -> crate::Result<()> {
        if let Some(encoder) = self.splitter.sub_encoder(self.metadata.as_ref(), record)? {
            encoder.encode_record_with_sym(record, symbol).await?;
        }
        Ok(())
    }
}

/// How to group records according to their index timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SplitDuration {
    /// Split by UTC day.
    Day,
    /// Split by Sunday-based weeks.
    Week,
    /// Split by month.
    Month,
}

/// Splits a stream by time.
#[derive(Debug)]
pub struct TimeSplitter<E, F> {
    build_encoder: F,
    split_duration: SplitDuration,
    encoders: HashMap<time::Date, E>,
}

/// Splits a stream by symbol.
///
/// It's generic over [`SymbolIndex`], allowing it to work with both
/// [`TsSymbolMap`](crate::TsSymbolMap) and [`PitSymbolMap`](crate::PitSymbolMap).
#[derive(Debug)]
pub struct SymbolSplitter<E, F, M> {
    build_encoder: F,
    encoders: HashMap<String, E>,
    symbol_map: M,
}

/// How to handle records with an rtype that doesn't map to a schema such as an
/// [`ErrorMsg`](crate::ErrorMsg).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum NoSchemaBehavior {
    /// Skip records with rtypes that have no schema.
    #[default]
    Skip,
    /// Return an error when encountering an rtype without a schema.
    Error,
    /// Route records with rtypes that have no schema to all existing encoders.
    Broadcast,
}

/// Splits a stream by schema.
#[derive(Debug)]
pub struct SchemaSplitter<E, F> {
    build_encoder: F,
    encoders: HashMap<Schema, E>,
    no_schema_behavior: NoSchemaBehavior,
}

impl<E, F> TimeSplitter<E, F>
where
    F: Fn(time::Date, Option<Metadata>) -> crate::Result<E>,
{
    /// Creates a new splitter that will split the input stream according to
    /// `split_duration`, creating a separate sub-encoder for each split using
    /// `build_encoder`.
    pub fn new(build_encoder: F, split_duration: SplitDuration) -> Self {
        Self {
            split_duration,
            build_encoder,
            encoders: HashMap::new(),
        }
    }

    fn split_metadata(
        split_duration: SplitDuration,
        mut metadata: Metadata,
        encoder_date: time::Date,
    ) -> Metadata {
        metadata.start = metadata
            .start()
            .max(encoder_date.with_time(Time::MIDNIGHT).assume_utc())
            .unix_timestamp_nanos() as u64;
        let end = match split_duration {
            SplitDuration::Day => encoder_date.next_day().unwrap(),
            SplitDuration::Week => encoder_date + time::Duration::days(7),
            SplitDuration::Month => {
                let end_year = if encoder_date.month() == time::Month::December {
                    encoder_date.year() + 1
                } else {
                    encoder_date.year()
                };
                encoder_date
                    .replace_month(encoder_date.month().next())
                    .unwrap()
                    .replace_year(end_year)
                    .unwrap()
            }
        }
        .with_time(Time::MIDNIGHT)
        .assume_utc();
        metadata.end = NonZeroU64::new(
            metadata
                .end()
                .map(|old_end| old_end.min(end))
                .unwrap_or(end)
                .unix_timestamp_nanos() as u64,
        );
        let start_date = metadata.start().date();
        let end = metadata.end().unwrap();
        let end_date = if end.time() == time::Time::MIDNIGHT {
            end.date()
        } else {
            end.date().next_day().unwrap()
        };
        metadata.mappings.retain_mut(|mapping| {
            mapping.intervals.retain_mut(|interval| {
                interval.start_date = interval.start_date.max(start_date);
                interval.end_date = interval.end_date.min(end_date);
                interval.start_date < end_date && interval.end_date > start_date
            });
            !mapping.intervals.is_empty()
        });
        let symbols = metadata
            .mappings
            .iter()
            .map(|m| &m.raw_symbol)
            .collect::<HashSet<_>>();
        metadata.symbols.retain(|s| symbols.contains(s));
        metadata.partial.retain(|s| symbols.contains(s));

        metadata
    }
}

impl<E, F> Splitter<E> for TimeSplitter<E, F>
where
    F: Fn(time::Date, Option<Metadata>) -> crate::Result<E>,
{
    fn sub_encoder<R>(
        &mut self,
        metadata: Option<&Metadata>,
        record: &R,
    ) -> crate::Result<Option<&mut E>>
    where
        R: Record,
    {
        use std::collections::hash_map::Entry;

        let index_date = record
            .index_date()
            .ok_or_else(|| crate::Error::encode("record has undefined timestamp"))?;
        let encoder_date = match self.split_duration {
            SplitDuration::Day => index_date,
            SplitDuration::Week if index_date.weekday() == Weekday::Sunday => index_date,
            SplitDuration::Week => index_date.prev_occurrence(Weekday::Sunday),
            SplitDuration::Month => index_date.replace_day(1).unwrap(),
        };
        let encoder = match self.encoders.entry(encoder_date) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let split_metadata = metadata
                    .cloned()
                    .map(|m| Self::split_metadata(self.split_duration, m, encoder_date));
                entry.insert((self.build_encoder)(encoder_date, split_metadata)?)
            }
        };
        Ok(Some(encoder))
    }

    fn sub_encoders<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
    where
        E: 'a,
    {
        self.encoders.values_mut()
    }
}

impl<E, F, M> SymbolSplitter<E, F, M>
where
    F: Fn(&str, Option<Metadata>) -> crate::Result<E>,
    M: SymbolIndex,
{
    /// Creates a new splitter that will split the input stream by symbol,
    /// creating a separate sub-encoder for each symbol using `build_encoder`.
    ///
    /// The `symbol_map` is used to look up the symbol for each record based on
    /// the instrument ID (and optionally the timestamp for `TsSymbolMap`).
    pub fn new(build_encoder: F, symbol_map: M) -> Self {
        Self {
            build_encoder,
            encoders: HashMap::new(),
            symbol_map,
        }
    }
}

impl<E, F, M> Splitter<E> for SymbolSplitter<E, F, M>
where
    F: Fn(&str, Option<Metadata>) -> crate::Result<E>,
    M: SymbolIndex,
{
    fn sub_encoder<R>(
        &mut self,
        metadata: Option<&Metadata>,
        record: &R,
    ) -> crate::Result<Option<&mut E>>
    where
        R: Record,
    {
        use std::collections::hash_map::Entry;

        let index_ts = record.index_ts();
        let symbol = self
            .symbol_map
            .get_for_rec(record)
            .ok_or_else(|| {
                crate::Error::encode(format!(
                    "no symbol mapping for instrument_id {} at {index_ts:?}",
                    record.header().instrument_id
                ))
            })?
            .clone();
        let encoder = match self.encoders.entry(symbol.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let split_metadata = metadata.cloned().map(|mut m| {
                    m.symbols.retain(|s| *s == symbol);
                    m.partial.retain(|s| *s == symbol);
                    m.mappings
                        .retain(|sym_mapping| sym_mapping.raw_symbol == symbol);
                    m
                });
                entry.insert((self.build_encoder)(&symbol, split_metadata)?)
            }
        };
        Ok(Some(encoder))
    }

    fn sub_encoders<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
    where
        E: 'a,
    {
        self.encoders.values_mut()
    }
}

impl<E, F> SchemaSplitter<E, F>
where
    F: Fn(Schema, Option<Metadata>) -> crate::Result<E>,
{
    /// Creates a new splitter that will split the input stream by schema,
    /// creating a separate sub-encoder for each schema using `build_encoder`.
    ///
    /// The `no_schema_behavior` determines how records with rtypes that don't map
    /// to a schema (such as [`ErrorMsg`](crate::ErrorMsg)) are handled.
    pub fn new(build_encoder: F, no_schema_behavior: NoSchemaBehavior) -> Self {
        Self {
            build_encoder,
            encoders: HashMap::new(),
            no_schema_behavior,
        }
    }
}

impl<E, F> Splitter<E> for SchemaSplitter<E, F>
where
    F: Fn(Schema, Option<Metadata>) -> crate::Result<E>,
    E: EncodeRecordRef,
{
    fn sub_encoder<R>(
        &mut self,
        metadata: Option<&Metadata>,
        record: &R,
    ) -> crate::Result<Option<&mut E>>
    where
        R: Record,
    {
        use std::collections::hash_map::Entry;

        let Some(schema) = RType::try_into_schema(record.header().rtype) else {
            return match self.no_schema_behavior {
                NoSchemaBehavior::Skip => Ok(None),
                NoSchemaBehavior::Error => Err(crate::Error::encode(format!(
                    "rtype {} has no corresponding schema",
                    record.header().rtype
                ))),
                NoSchemaBehavior::Broadcast => {
                    let rec_ref =
                    // SAFETY: `record` is a valid DBN record: it satisfies `R: Record`.
                        unsafe { RecordRef::unchecked_from_header(record.header() as *const _) };
                    for encoder in self.encoders.values_mut() {
                        // Have to use `encode_record_ref` here because `SplitEncoder` supports
                        // both `EncodeRecord` and `EncodeRecordRef`
                        encoder.encode_record_ref(rec_ref)?;
                    }
                    Ok(None)
                }
            };
        };
        let encoder = match self.encoders.entry(schema) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let split_metadata = metadata.cloned().map(|mut m| {
                    // Only set schema if not broadcasting (broadcast outputs contain mixed rtypes)
                    if self.no_schema_behavior != NoSchemaBehavior::Broadcast {
                        m.schema = Some(schema);
                    } else {
                        m.schema = None;
                    }
                    m
                });
                entry.insert((self.build_encoder)(schema, split_metadata)?)
            }
        };
        Ok(Some(encoder))
    }

    fn sub_encoders<'a>(&'a mut self) -> impl Iterator<Item = &'a mut E>
    where
        E: 'a,
    {
        self.encoders.values_mut()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use time::macros::{date, datetime};

    use super::*;
    use crate::{rtype, MboMsg, Mbp1Msg, RecordHeader, TradeMsg, TsSymbolMap, UNDEF_TIMESTAMP};

    /// Helper to create an MboMsg with a specific timestamp and instrument_id
    fn mbo_msg(ts: u64, instrument_id: u32) -> MboMsg {
        MboMsg {
            hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, instrument_id, ts),
            ts_recv: ts,
            ..Default::default()
        }
    }

    /// Helper to create a TradeMsg with a specific timestamp and instrument_id
    fn trade_msg(ts: u64, instrument_id: u32) -> TradeMsg {
        TradeMsg {
            hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, instrument_id, ts),
            ts_recv: ts,
            ..Default::default()
        }
    }

    /// Helper to create an Mbp1Msg with a specific timestamp and instrument_id
    fn mbp1_msg(ts: u64, instrument_id: u32) -> Mbp1Msg {
        Mbp1Msg {
            hd: RecordHeader::new::<Mbp1Msg>(rtype::MBP_1, 1, instrument_id, ts),
            ts_recv: ts,
            ..Default::default()
        }
    }

    /// Simple test encoder that just stores the records it receives
    #[derive(Debug, Default)]
    struct TestEncoder {
        records: Vec<(u64, u32)>, // (ts_event, instrument_id)
    }

    impl EncodeRecord for TestEncoder {
        fn encode_record<R: DbnEncodable>(&mut self, record: &R) -> crate::Result<()> {
            self.records
                .push((record.header().ts_event, record.header().instrument_id));
            Ok(())
        }

        fn flush(&mut self) -> crate::Result<()> {
            Ok(())
        }
    }

    impl EncodeRecordRef for TestEncoder {
        fn encode_record_ref(&mut self, record: RecordRef) -> crate::Result<()> {
            self.records
                .push((record.header().ts_event, record.header().instrument_id));
            Ok(())
        }

        unsafe fn encode_record_ref_ts_out(
            &mut self,
            record: RecordRef,
            _ts_out: bool,
        ) -> crate::Result<()> {
            self.encode_record_ref(record)
        }
    }

    type TestTimeSplitter =
        TimeSplitter<TestEncoder, fn(time::Date, Option<Metadata>) -> crate::Result<TestEncoder>>;

    #[test]
    fn test_time_splitter_by_day_single_day() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = TimeSplitter::new(build_encoder, SplitDuration::Day);

        let ts1 = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts2 = datetime!(2023-07-15 14:00 UTC).unix_timestamp_nanos() as u64;
        let ts3 = datetime!(2023-07-15 18:00 UTC).unix_timestamp_nanos() as u64;

        let rec1 = mbo_msg(ts1, 100);
        let rec2 = mbo_msg(ts2, 101);
        let rec3 = mbo_msg(ts3, 102);

        splitter.sub_encoder(None, &rec1).unwrap();
        splitter.sub_encoder(None, &rec2).unwrap();
        splitter.sub_encoder(None, &rec3).unwrap();

        assert_eq!(splitter.encoders.len(), 1);
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 15)));
    }

    #[test]
    fn test_time_splitter_by_day_multiple_days() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = TimeSplitter::new(build_encoder, SplitDuration::Day);

        let ts_day1 = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_day2 = datetime!(2023-07-16 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_day3 = datetime!(2023-07-17 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec1 = mbo_msg(ts_day1, 100);
        let rec2 = mbo_msg(ts_day2, 100);
        let rec3 = mbo_msg(ts_day3, 100);

        splitter.sub_encoder(None, &rec1).unwrap();
        splitter.sub_encoder(None, &rec2).unwrap();
        splitter.sub_encoder(None, &rec3).unwrap();

        assert_eq!(splitter.encoders.len(), 3);
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 15)));
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 16)));
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 17)));
    }

    #[test]
    fn test_time_splitter_by_week() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = TimeSplitter::new(build_encoder, SplitDuration::Week);

        // 2023-07-16 is a Sunday, 2023-07-17 is a Monday
        let ts_sun = datetime!(2023-07-16 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_mon = datetime!(2023-07-17 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_tue = datetime!(2023-07-18 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_next_sun = datetime!(2023-07-23 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec_sun = mbo_msg(ts_sun, 100);
        let rec_mon = mbo_msg(ts_mon, 100);
        let rec_tue = mbo_msg(ts_tue, 100);
        let rec_next_sun = mbo_msg(ts_next_sun, 100);

        splitter.sub_encoder(None, &rec_sun).unwrap();
        splitter.sub_encoder(None, &rec_mon).unwrap();
        splitter.sub_encoder(None, &rec_tue).unwrap();
        splitter.sub_encoder(None, &rec_next_sun).unwrap();

        // Sunday is its own week start, Mon-Sat belong to that Sunday
        // Next Sunday is a new week
        assert_eq!(splitter.encoders.len(), 2);
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 16))); // First Sunday
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 23))); // Next Sunday
    }

    #[test]
    fn test_time_splitter_by_month() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = TimeSplitter::new(build_encoder, SplitDuration::Month);

        let ts_jul = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_aug = datetime!(2023-08-10 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_sep = datetime!(2023-09-05 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec_jul = mbo_msg(ts_jul, 100);
        let rec_aug = mbo_msg(ts_aug, 100);
        let rec_sep = mbo_msg(ts_sep, 100);

        splitter.sub_encoder(None, &rec_jul).unwrap();
        splitter.sub_encoder(None, &rec_aug).unwrap();
        splitter.sub_encoder(None, &rec_sep).unwrap();

        assert_eq!(splitter.encoders.len(), 3);
        assert!(splitter.encoders.contains_key(&date!(2023 - 07 - 01)));
        assert!(splitter.encoders.contains_key(&date!(2023 - 08 - 01)));
        assert!(splitter.encoders.contains_key(&date!(2023 - 09 - 01)));
    }

    #[test]
    fn test_time_splitter_by_month_year_boundary() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = TimeSplitter::new(build_encoder, SplitDuration::Month);

        let ts_dec = datetime!(2023-12-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_jan = datetime!(2024-01-10 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec_dec = mbo_msg(ts_dec, 100);
        let rec_jan = mbo_msg(ts_jan, 100);

        splitter.sub_encoder(None, &rec_dec).unwrap();
        splitter.sub_encoder(None, &rec_jan).unwrap();

        assert_eq!(splitter.encoders.len(), 2);
        assert!(splitter.encoders.contains_key(&date!(2023 - 12 - 01)));
        assert!(splitter.encoders.contains_key(&date!(2024 - 01 - 01)));
    }

    #[test]
    fn test_symbol_splitter_multiple_symbols() {
        let mut symbol_map = TsSymbolMap::new();
        symbol_map
            .insert(
                100,
                date!(2023 - 07 - 01),
                date!(2023 - 08 - 01),
                Arc::new("AAPL".to_owned()),
            )
            .unwrap();
        symbol_map
            .insert(
                101,
                date!(2023 - 07 - 01),
                date!(2023 - 08 - 01),
                Arc::new("TSLA".to_owned()),
            )
            .unwrap();
        symbol_map
            .insert(
                102,
                date!(2023 - 07 - 01),
                date!(2023 - 08 - 01),
                Arc::new("MSFT".to_owned()),
            )
            .unwrap();

        let build_encoder = |_symbol: &str, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = SymbolSplitter::new(build_encoder, symbol_map);

        let ts = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec_aapl = mbo_msg(ts, 100);
        let rec_tsla = mbo_msg(ts, 101);
        let rec_msft = mbo_msg(ts, 102);

        splitter.sub_encoder(None, &rec_aapl).unwrap();
        splitter.sub_encoder(None, &rec_tsla).unwrap();
        splitter.sub_encoder(None, &rec_msft).unwrap();

        assert_eq!(splitter.encoders.len(), 3);
        assert!(splitter.encoders.keys().any(|k| k == "AAPL"));
        assert!(splitter.encoders.keys().any(|k| k == "TSLA"));
        assert!(splitter.encoders.keys().any(|k| k == "MSFT"));
    }

    #[test]
    fn test_symbol_splitter_same_symbol_multiple_records() {
        let mut symbol_map = TsSymbolMap::new();
        symbol_map
            .insert(
                100,
                date!(2023 - 07 - 01),
                date!(2023 - 08 - 01),
                Arc::new("AAPL".to_owned()),
            )
            .unwrap();

        let build_encoder = |_symbol: &str, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = SymbolSplitter::new(build_encoder, symbol_map);

        let ts1 = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts2 = datetime!(2023-07-15 11:00 UTC).unix_timestamp_nanos() as u64;
        let ts3 = datetime!(2023-07-15 12:00 UTC).unix_timestamp_nanos() as u64;

        let rec1 = mbo_msg(ts1, 100);
        let rec2 = mbo_msg(ts2, 100);
        let rec3 = mbo_msg(ts3, 100);

        splitter.sub_encoder(None, &rec1).unwrap();
        splitter.sub_encoder(None, &rec2).unwrap();
        splitter.sub_encoder(None, &rec3).unwrap();

        assert_eq!(splitter.encoders.len(), 1);
    }

    #[test]
    fn test_symbol_splitter_unknown_instrument() {
        let mut symbol_map = TsSymbolMap::new();
        symbol_map
            .insert(
                100,
                date!(2023 - 07 - 01),
                date!(2023 - 08 - 01),
                Arc::new("AAPL".to_owned()),
            )
            .unwrap();

        let build_encoder = |_symbol: &str, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = SymbolSplitter::new(build_encoder, symbol_map);

        let ts = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;

        // instrument ID not in symbol map
        let rec_unknown = mbo_msg(ts, 999);

        let result = splitter.sub_encoder(None, &rec_unknown);
        assert!(result.is_err());
        assert_eq!(splitter.encoders.len(), 0);
    }

    #[test]
    fn test_schema_splitter_multiple_schemas() {
        let build_encoder =
            |_schema: Schema, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = SchemaSplitter::new(build_encoder, NoSchemaBehavior::Skip);

        let ts = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec_mbo = mbo_msg(ts, 100);
        let rec_trades = trade_msg(ts, 100);
        let rec_mbp1 = mbp1_msg(ts, 100);

        splitter.sub_encoder(None, &rec_mbo).unwrap();
        splitter.sub_encoder(None, &rec_trades).unwrap();
        splitter.sub_encoder(None, &rec_mbp1).unwrap();

        assert_eq!(splitter.encoders.len(), 3);
        assert!(splitter.encoders.contains_key(&Schema::Mbo));
        assert!(splitter.encoders.contains_key(&Schema::Trades));
        assert!(splitter.encoders.contains_key(&Schema::Mbp1));
    }

    #[test]
    fn test_schema_splitter_same_schema() {
        let build_encoder =
            |_schema: Schema, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = SchemaSplitter::new(build_encoder, NoSchemaBehavior::Skip);

        let ts1 = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts2 = datetime!(2023-07-15 11:00 UTC).unix_timestamp_nanos() as u64;

        let rec1 = mbo_msg(ts1, 100);
        let rec2 = mbo_msg(ts2, 101);

        splitter.sub_encoder(None, &rec1).unwrap();
        splitter.sub_encoder(None, &rec2).unwrap();

        // Both are MBO, so should have only one encoder
        assert_eq!(splitter.encoders.len(), 1);
        assert!(splitter.encoders.contains_key(&Schema::Mbo));
    }

    #[test]
    fn test_split_encoder_with_time_splitter() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let splitter = TimeSplitter::new(build_encoder, SplitDuration::Day);
        let mut encoder: SplitEncoder<_, TestEncoder> = SplitEncoder::records_only(splitter);

        let ts_day1 = datetime!(2023-07-15 10:00 UTC).unix_timestamp_nanos() as u64;
        let ts_day2 = datetime!(2023-07-16 10:00 UTC).unix_timestamp_nanos() as u64;

        let rec1 = mbo_msg(ts_day1, 100);
        let rec2 = mbo_msg(ts_day1, 101);
        let rec3 = mbo_msg(ts_day2, 100);

        encoder.encode_record(&rec1).unwrap();
        encoder.encode_record(&rec2).unwrap();
        encoder.encode_record(&rec3).unwrap();

        let day1_encoder = encoder
            .splitter
            .encoders
            .get(&date!(2023 - 07 - 15))
            .unwrap();
        let day2_encoder = encoder
            .splitter
            .encoders
            .get(&date!(2023 - 07 - 16))
            .unwrap();

        assert_eq!(day1_encoder.records.len(), 2);
        assert_eq!(day2_encoder.records.len(), 1);
    }

    #[test]
    fn test_split_metadata_by_day() {
        use crate::{MappingInterval, MetadataBuilder, SType, Schema, SymbolMapping};
        use std::num::NonZeroU64;

        let metadata = MetadataBuilder::new()
            .dataset("TEST".to_owned())
            .schema(Some(Schema::Mbo))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .start(datetime!(2023-07-01 00:00 UTC).unix_timestamp_nanos() as u64)
            .end(NonZeroU64::new(
                datetime!(2023-07-10 00:00 UTC).unix_timestamp_nanos() as u64,
            ))
            .symbols(vec!["AAPL".to_owned()])
            .mappings(vec![SymbolMapping {
                raw_symbol: "AAPL".to_owned(),
                intervals: vec![MappingInterval {
                    start_date: date!(2023 - 07 - 01),
                    end_date: date!(2023 - 07 - 10),
                    symbol: "100".to_owned(),
                }],
            }])
            .build();

        let split_meta = TestTimeSplitter::split_metadata(
            SplitDuration::Day,
            metadata.clone(),
            date!(2023 - 07 - 05),
        );

        assert_eq!(
            split_meta.start,
            datetime!(2023-07-05 00:00 UTC).unix_timestamp_nanos() as u64
        );
        assert_eq!(
            split_meta.end.unwrap().get(),
            datetime!(2023-07-06 00:00 UTC).unix_timestamp_nanos() as u64
        );
    }

    #[test]
    fn test_split_metadata_by_month() {
        use crate::{MappingInterval, MetadataBuilder, SType, Schema, SymbolMapping};
        use std::num::NonZeroU64;

        let metadata = MetadataBuilder::new()
            .dataset("TEST".to_owned())
            .schema(Some(Schema::Mbo))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .start(datetime!(2023-06-15 00:00 UTC).unix_timestamp_nanos() as u64)
            .end(NonZeroU64::new(
                datetime!(2023-08-15 00:00 UTC).unix_timestamp_nanos() as u64,
            ))
            .symbols(vec!["AAPL".to_owned()])
            .mappings(vec![SymbolMapping {
                raw_symbol: "AAPL".to_owned(),
                intervals: vec![MappingInterval {
                    start_date: date!(2023 - 06 - 15),
                    end_date: date!(2023 - 08 - 15),
                    symbol: "100".to_owned(),
                }],
            }])
            .build();

        // Test metadata splitting for July
        let split_meta = TestTimeSplitter::split_metadata(
            SplitDuration::Month,
            metadata.clone(),
            date!(2023 - 07 - 01),
        );

        // Check that start/end are correctly bounded to July
        assert_eq!(
            split_meta.start,
            datetime!(2023-07-01 00:00 UTC).unix_timestamp_nanos() as u64
        );
        assert_eq!(
            split_meta.end.unwrap().get(),
            datetime!(2023-08-01 00:00 UTC).unix_timestamp_nanos() as u64
        );
    }

    #[test]
    fn test_split_metadata_retains_relevant_mappings() {
        use crate::{MappingInterval, MetadataBuilder, SType, Schema, SymbolMapping};
        use std::num::NonZeroU64;

        let metadata = MetadataBuilder::new()
            .dataset("TEST".to_owned())
            .schema(Some(Schema::Mbo))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .start(datetime!(2023-07-01 00:00 UTC).unix_timestamp_nanos() as u64)
            .end(NonZeroU64::new(
                datetime!(2023-07-31 00:00 UTC).unix_timestamp_nanos() as u64,
            ))
            .symbols(vec!["AAPL".to_owned(), "TSLA".to_owned()])
            .mappings(vec![
                SymbolMapping {
                    raw_symbol: "AAPL".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date: date!(2023 - 07 - 01),
                        end_date: date!(2023 - 07 - 15),
                        symbol: "100".to_owned(),
                    }],
                },
                SymbolMapping {
                    raw_symbol: "TSLA".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date: date!(2023 - 07 - 10),
                        end_date: date!(2023 - 07 - 25),
                        symbol: "101".to_owned(),
                    }],
                },
            ])
            .build();

        // both AAPL and TSLA should be present
        let split_meta = TestTimeSplitter::split_metadata(
            SplitDuration::Day,
            metadata.clone(),
            date!(2023 - 07 - 12),
        );
        assert_eq!(split_meta.mappings.len(), 2);
        assert_eq!(split_meta.symbols.len(), 2);

        // only AAPL should be present
        let split_meta = TestTimeSplitter::split_metadata(
            SplitDuration::Day,
            metadata.clone(),
            date!(2023 - 07 - 05),
        );
        assert_eq!(split_meta.mappings.len(), 1);
        assert_eq!(split_meta.mappings[0].raw_symbol, "AAPL");
        assert_eq!(split_meta.symbols.len(), 1);

        // only TSLA should be present
        let split_meta = TestTimeSplitter::split_metadata(
            SplitDuration::Day,
            metadata.clone(),
            date!(2023 - 07 - 20),
        );
        assert_eq!(split_meta.mappings.len(), 1);
        assert_eq!(split_meta.mappings[0].raw_symbol, "TSLA");
        assert_eq!(split_meta.symbols.len(), 1);
    }

    #[test]
    fn test_record_with_undef_timestamp_returns_error() {
        let build_encoder =
            |_date: time::Date, _metadata: Option<Metadata>| Ok(TestEncoder::default());
        let mut splitter = TimeSplitter::new(build_encoder, SplitDuration::Day);

        let rec = mbo_msg(UNDEF_TIMESTAMP, 100);
        splitter.sub_encoder(None, &rec).unwrap_err();
    }
}
