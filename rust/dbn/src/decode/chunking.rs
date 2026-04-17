//! Filtering and byte-chunking of DBN record streams.
//!
//! [`RecordFilter`] drops records failing header filters (`instrument_id`,
//! `publisher_id`) or a half-open time window `[start_ts, end_ts)`, and is
//! itself a [`DecodeRecordRef`] so it composes into any decoder pipeline.
//! [`ByteChunker`] aggregates records from any [`DecodeRecordRef`] into
//! bounded-size [`ByteChunk`]s of raw record bytes.
//!
//! Both wrappers are transparent to [`VersionUpgradePolicy`](crate::VersionUpgradePolicy):
//! the bytes the underlying decoder hands back are what land in each chunk,
//! so a decoder configured to upgrade V1 input to V3 yields V3 record bytes.
//!
//! # Example
//!
//! ```no_run
//! use dbn::VersionUpgradePolicy;
//! use dbn::decode::{ByteChunker, DbnMetadata, DynDecoder, RecordFilter};
//! use dbn::encode::DbnMetadataEncoder;
//!
//! let decoder = DynDecoder::from_file("20241007.dbn.zst", VersionUpgradePolicy::UpgradeToV3)?;
//! let metadata = decoder.metadata().clone();
//! let filter = RecordFilter::builder(decoder).instrument_ids([123_456]).build()?;
//! let mut chunker = ByteChunker::new(filter);
//!
//! let mut out: Vec<u8> = Vec::new();
//! DbnMetadataEncoder::new(&mut out).encode(&metadata)?;
//! while let Some(chunk) = chunker.next_chunk()? {
//!     out.extend_from_slice(chunk.bytes());
//! }
//! # Ok::<(), dbn::Error>(())
//! ```
use std::{fmt, num::NonZeroUsize};

use crate::{
    decode::{
        private, DbnMetadata, DecodeRecord, DecodeRecordRef, DecodeStream, StreamIterDecoder,
    },
    HasRType, Metadata, Record, RecordBuf, RecordRef,
};

/// Decision produced by [`FilterState::classify`] for a single record.
enum Classification {
    Emit,
    DropTime,
    DropInstrument,
    DropPublisher,
    End,
}

struct FilterState {
    start_ts: Option<u64>,
    end_ts: Option<u64>,
    // Empty `Vec` means "no filter". Vec rather than `Option<Vec>` so the
    // unfiltered default doesn't allocate.
    instrument_ids: Vec<u32>,
    publisher_ids: Vec<u16>,
    // Latched: set once when `end_ts` trips and never cleared; `done` is set
    // in the same pass so no later record can overwrite it.
    tripping_record: Option<RecordBuf>,
    records_emitted: u64,
    records_dropped_by_time: u64,
    records_dropped_by_instrument_id: u64,
    records_dropped_by_publisher_id: u64,
}

impl FilterState {
    fn passes_instrument(&self, id: u32) -> bool {
        self.instrument_ids.is_empty() || self.instrument_ids.contains(&id)
    }

    fn passes_publisher(&self, id: u16) -> bool {
        self.publisher_ids.is_empty() || self.publisher_ids.contains(&id)
    }

    /// Classifies `rec` against every filter, updating the drop counters and
    /// the tripping-record slot as a side effect.
    fn classify(&mut self, rec: RecordRef<'_>) -> Classification {
        let primary_ts = rec.raw_index_ts();
        let hd = rec.header();

        if let Some(end) = self.end_ts {
            if primary_ts >= end {
                // Only stash the tripping record if it would have passed the
                // header filters, otherwise a filtered-out record could leak
                // into a resumed stream via the inner decoder.
                if self.passes_instrument(hd.instrument_id)
                    && self.passes_publisher(hd.publisher_id)
                {
                    self.tripping_record = Some(rec.to_owned());
                }
                return Classification::End;
            }
        }
        if let Some(start) = self.start_ts {
            if primary_ts < start {
                self.records_dropped_by_time += 1;
                return Classification::DropTime;
            }
        }
        if !self.passes_instrument(hd.instrument_id) {
            self.records_dropped_by_instrument_id += 1;
            return Classification::DropInstrument;
        }
        if !self.passes_publisher(hd.publisher_id) {
            self.records_dropped_by_publisher_id += 1;
            return Classification::DropPublisher;
        }

        self.records_emitted += 1;
        Classification::Emit
    }
}

/// Wraps a [`DecodeRecordRef`] and drops records failing its header or time
/// filters. Implements [`DecodeRecordRef`] and composes with [`ByteChunker`]
/// or anything else that accepts a decoder.
///
/// When iteration terminates on `end_ts`, the tripping record is stashed in
/// [`tripping_record`](Self::tripping_record). Unwrap with
/// [`into_parts`](Self::into_parts) to recover both the decoder and the
/// tripping record for a seamless resume. Tripping records that would have
/// failed the header filters are dropped rather than stashed.
pub struct RecordFilter<D> {
    decoder: D,
    done: bool,
    state: FilterState,
}

impl<D> RecordFilter<D> {
    /// Returns a builder for configuring a [`RecordFilter`]. Call
    /// [`build`](RecordFilterBuilder::build) with no setters invoked for a
    /// passthrough that lets every record through.
    pub fn builder(decoder: D) -> RecordFilterBuilder<D> {
        RecordFilterBuilder::new(decoder)
    }

    /// Immutable reference to the wrapped decoder.
    pub fn get_ref(&self) -> &D {
        &self.decoder
    }

    /// Mutable reference to the wrapped decoder.
    pub fn get_mut(&mut self) -> &mut D {
        &mut self.decoder
    }

    /// Consumes the filter and returns the wrapped decoder together with the
    /// stashed tripping record (`None` unless iteration was terminated by
    /// `end_ts` and the tripping record passed the header filters). The
    /// decoder is positioned after the last record the filter pulled from it,
    /// so prepending the tripping record's bytes to subsequent output yields
    /// a seamless resume.
    pub fn into_parts(self) -> (D, Option<RecordBuf>) {
        (self.decoder, self.state.tripping_record)
    }

    /// If iteration was terminated by `end_ts`, returns the tripping record
    /// without consuming the filter. Returns `None` if iteration ended
    /// naturally or has not yet terminated.
    pub fn tripping_record(&self) -> Option<&RecordBuf> {
        self.state.tripping_record.as_ref()
    }

    /// Snapshot of the emit/drop counters. The `end_ts` tripping record is
    /// not counted in any `dropped_by_*` field; retrieve it via
    /// [`tripping_record`](Self::tripping_record) instead.
    pub fn stats(&self) -> FilterStats {
        FilterStats {
            emitted: self.state.records_emitted,
            dropped_by_time: self.state.records_dropped_by_time,
            dropped_by_instrument_id: self.state.records_dropped_by_instrument_id,
            dropped_by_publisher_id: self.state.records_dropped_by_publisher_id,
        }
    }
}

/// Emit/drop counters for a [`RecordFilter`]. Returned by
/// [`RecordFilter::stats`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub struct FilterStats {
    /// Records that passed every filter.
    pub emitted: u64,
    /// Records dropped because their primary timestamp was below `start_ts`.
    /// The `end_ts` tripping record is not counted here.
    pub dropped_by_time: u64,
    /// Records dropped because their `instrument_id` was not in the configured
    /// allow-list.
    pub dropped_by_instrument_id: u64,
    /// Records dropped because their `publisher_id` was not in the configured
    /// allow-list.
    pub dropped_by_publisher_id: u64,
}

impl<D: DbnMetadata> DbnMetadata for RecordFilter<D> {
    fn metadata(&self) -> &Metadata {
        self.decoder.metadata()
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        self.decoder.metadata_mut()
    }
}

impl<D: DecodeRecordRef + private::LastRecord> DecodeRecordRef for RecordFilter<D> {
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
        if self.done {
            return Ok(None);
        }
        loop {
            let rec = match self.decoder.decode_record_ref() {
                Ok(Some(r)) => r,
                Ok(None) => {
                    self.done = true;
                    return Ok(None);
                }
                Err(e) => {
                    self.done = true;
                    return Err(e);
                }
            };
            match self.state.classify(rec) {
                Classification::Emit => break,
                Classification::End => {
                    self.done = true;
                    return Ok(None);
                }
                Classification::DropTime
                | Classification::DropInstrument
                | Classification::DropPublisher => continue,
            }
        }
        Ok(self.decoder.last_record())
    }
}

impl<D: DecodeRecordRef + private::LastRecord> DecodeRecord for RecordFilter<D> {
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decode_record_ref().and_then(|rec| {
            if let Some(rec) = rec {
                rec.try_get().map(Some)
            } else {
                Ok(None)
            }
        })
    }
}

impl<D: private::LastRecord> private::LastRecord for RecordFilter<D> {
    // After a successful `decode_record_ref` returning `Some`, the inner
    // decoder's last record is the one we emitted: the loop advances the
    // decoder through each dropped record and terminates on the emit, so
    // the inner decoder is positioned at the emitted record. `End` and
    // drain both return `Ok(None)` from `decode_record_ref`, in which case
    // `StreamIterDecoder::get` short-circuits and never calls this.
    fn last_record(&self) -> Option<RecordRef<'_>> {
        self.decoder.last_record()
    }
}

impl<D: private::LastRecord + DecodeRecordRef> DecodeStream for RecordFilter<D> {
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        StreamIterDecoder::new(self)
    }
}

impl<D> fmt::Debug for RecordFilter<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RecordFilter")
            .field("start_ts", &self.state.start_ts)
            .field("end_ts", &self.state.end_ts)
            .field("instrument_ids", &self.state.instrument_ids)
            .field("publisher_ids", &self.state.publisher_ids)
            .field("stats", &self.stats())
            .finish_non_exhaustive()
    }
}

/// Helper for configuring a [`RecordFilter`].
pub struct RecordFilterBuilder<D> {
    decoder: D,
    start_ts: Option<u64>,
    end_ts: Option<u64>,
    instrument_ids: Vec<u32>,
    publisher_ids: Vec<u16>,
}

impl<D> RecordFilterBuilder<D> {
    /// Creates a new builder wrapping `decoder` with no filters configured.
    pub fn new(decoder: D) -> Self {
        Self {
            decoder,
            start_ts: None,
            end_ts: None,
            instrument_ids: Vec::new(),
            publisher_ids: Vec::new(),
        }
    }

    /// Restricts output to records whose `instrument_id` is in `ids`. An
    /// empty iterator means no filter. Calling again replaces the previous
    /// list rather than extending it.
    ///
    /// Filters are matched with `Vec::contains`, which is faster than a
    /// `HashSet` lookup at the list sizes this API is designed for (on the
    /// order of tens of ids). Callers filtering against thousands of ids
    /// should pre-filter upstream.
    pub fn instrument_ids<I>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = u32>,
    {
        self.instrument_ids = ids.into_iter().collect();
        self
    }

    /// Restricts output to records whose `publisher_id` is in `ids`. Accepts
    /// any iterator whose items can be converted to `u16`, so either raw ids
    /// or [`Publisher`](crate::Publisher) values work. An empty iterator
    /// means no filter. Calling again replaces the previous list rather
    /// than extending it. See [`instrument_ids`](Self::instrument_ids) for
    /// notes on filter-list sizing.
    pub fn publisher_ids<I, P>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<u16>,
    {
        self.publisher_ids = ids.into_iter().map(Into::into).collect();
        self
    }

    /// Lower bound of the half-open time window `[start_ts, end_ts)`. Skips
    /// records whose primary timestamp is less than `start_ts`. The primary
    /// timestamp is [`Record::raw_index_ts`], which matches the sort order of
    /// DBN files: `ts_recv` for every schema that carries it (MBO, MBP,
    /// trades, BBO, CMBP, CBBO, status, etc.), with a fallback to `ts_event`
    /// for schemas without a `ts_recv` field.
    ///
    /// Records whose `rtype` isn't recognized by this crate (e.g. a schema
    /// added after this version) also fall back to `ts_event`, since the
    /// dispatch keyed on `rtype` can't resolve the right field. Streams mixing
    /// known and unknown rtypes are filtered consistently within each group
    /// but not across the boundary.
    ///
    /// Unlike [`end_ts`](Self::end_ts), the lower bound checks every record
    /// individually and does not require the input to be ordered by the
    /// primary timestamp.
    pub fn start_ts(mut self, start_ts: u64) -> Self {
        self.start_ts = Some(start_ts);
        self
    }

    /// Upper bound of the half-open time window `[start_ts, end_ts)`.
    /// Terminates iteration at the first record whose primary timestamp is at
    /// or past `end_ts`. The primary timestamp is [`Record::raw_index_ts`];
    /// see [`start_ts`](Self::start_ts) for the field it resolves to per
    /// schema.
    ///
    /// The tripping record is consumed from the decoder and *not* emitted in
    /// any chunk. If it would have passed the other header filters, it is
    /// stashed on the [`RecordFilter`] so callers can resume iteration
    /// without losing it; otherwise the tripping record is dropped.
    ///
    /// `end_ts` is checked before the other filters, so a record at or past
    /// `end_ts` terminates iteration even when a filter would otherwise have
    /// dropped it.
    ///
    /// Early termination requires the input to be monotonically non-decreasing
    /// in the primary timestamp. DBN files produced by the Databento API
    /// satisfy this; custom inputs that re-sort by a different field (e.g. a
    /// cross-venue merge keyed on `ts_event` when `raw_index_ts` is `ts_recv`)
    /// will silently drop records past the first trip. For such inputs,
    /// either filter upstream by timestamp or omit `end_ts` and filter the
    /// emitted chunks externally.
    pub fn end_ts(mut self, end_ts: u64) -> Self {
        self.end_ts = Some(end_ts);
        self
    }

    /// Builds a [`RecordFilter`] with the configured settings.
    ///
    /// # Errors
    /// This function returns an error if `start_ts > end_ts`.
    pub fn build(self) -> crate::Result<RecordFilter<D>> {
        if let (Some(s), Some(e)) = (self.start_ts, self.end_ts) {
            if s > e {
                return Err(crate::Error::BadArgument {
                    param_name: "start_ts".to_owned(),
                    desc: "must be less than or equal to end_ts".to_owned(),
                });
            }
        }
        Ok(RecordFilter {
            decoder: self.decoder,
            done: false,
            state: FilterState {
                start_ts: self.start_ts,
                end_ts: self.end_ts,
                instrument_ids: self.instrument_ids,
                publisher_ids: self.publisher_ids,
                tripping_record: None,
                records_emitted: 0,
                records_dropped_by_time: 0,
                records_dropped_by_instrument_id: 0,
                records_dropped_by_publisher_id: 0,
            },
        })
    }
}

/// One chunk of DBN record bytes yielded by a [`ByteChunker`].
///
/// The slice returned by [`bytes`](Self::bytes) borrows from the chunker's
/// internal buffer and is only valid until the next `next_chunk` call. Size
/// bounds are soft: `max_bytes` may be exceeded by up to one record's size.
pub struct ByteChunk<'a> {
    bytes: &'a [u8],
    count: u64,
}

impl<'a> ByteChunk<'a> {
    /// Concatenated bytes of one or more complete DBN records.
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }

    /// Number of records contained in [`bytes`](Self::bytes).
    pub fn count(&self) -> u64 {
        self.count
    }
}

impl fmt::Debug for ByteChunk<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ByteChunk")
            .field("count", &self.count)
            .field("len", &self.bytes.len())
            .finish()
    }
}

/// Aggregates records from a [`DecodeRecordRef`] into bounded byte chunks.
/// Each chunk is a contiguous slice of complete DBN records in the version
/// produced by the wrapped decoder, so prepending matching metadata gives a
/// valid DBN stream. Compose with [`RecordFilter`] to filter before chunking.
///
/// The internal buffer is reused between chunks and its capacity settles near
/// `max_bytes` after the first full chunk; that capacity is released when the
/// chunker is dropped.
pub struct ByteChunker<D> {
    decoder: D,
    max_records: Option<NonZeroUsize>,
    max_bytes: NonZeroUsize,
    // Cleared (not shrunk) between chunks. Capacity settles near `max_bytes`
    // after the first full chunk and stays there for the chunker's lifetime,
    // trading retained memory for predictable allocation behavior.
    buf: Vec<u8>,
    done: bool,
}

impl<D> ByteChunker<D> {
    /// The default soft byte ceiling for each chunk, 4 MiB. Sized to give a
    /// predictable peak buffer size across schemas; across DBN record sizes
    /// this buffers anywhere from roughly 11k [`Mbp10Msg`](crate::Mbp10Msg)
    /// to 75k [`MboMsg`](crate::MboMsg) per chunk.
    ///
    /// There is no default record-count cap; set one via
    /// [`with_max_records`](Self::with_max_records) if you need one.
    pub const DEFAULT_MAX_BYTES: NonZeroUsize = NonZeroUsize::new(4 * 1024 * 1024).unwrap();

    /// Wraps `decoder` with the default byte ceiling and no record cap.
    pub fn new(decoder: D) -> Self {
        Self {
            decoder,
            max_records: None,
            max_bytes: Self::DEFAULT_MAX_BYTES,
            buf: Vec::new(),
            done: false,
        }
    }

    /// Sets a hard record-count ceiling per chunk. No default.
    pub fn with_max_records(mut self, n: NonZeroUsize) -> Self {
        self.max_records = Some(n);
        self
    }

    /// Sets a soft byte ceiling per chunk, overriding
    /// [`DEFAULT_MAX_BYTES`](Self::DEFAULT_MAX_BYTES). Once the running chunk
    /// reaches or exceeds this size, the chunk is closed.
    pub fn with_max_bytes(mut self, n: NonZeroUsize) -> Self {
        self.max_bytes = n;
        self
    }

    /// Immutable reference to the wrapped decoder.
    pub fn get_ref(&self) -> &D {
        &self.decoder
    }

    /// Mutable reference to the wrapped decoder.
    pub fn get_mut(&mut self) -> &mut D {
        &mut self.decoder
    }

    /// Consumes the chunker and returns the wrapped decoder.
    pub fn into_inner(self) -> D {
        self.decoder
    }
}

impl<D: DbnMetadata> DbnMetadata for ByteChunker<D> {
    fn metadata(&self) -> &Metadata {
        self.decoder.metadata()
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        self.decoder.metadata_mut()
    }
}

impl<D> fmt::Debug for ByteChunker<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ByteChunker")
            .field("max_records", &self.max_records)
            .field("max_bytes", &self.max_bytes)
            .finish_non_exhaustive()
    }
}

impl<D: DecodeRecordRef> ByteChunker<D> {
    /// Pulls records from the decoder until a chunk is full or the decoder
    /// is exhausted. Returns `Ok(None)` when there are no more records to
    /// emit. Subsequent calls after drain continue to return `Ok(None)`
    /// without touching the underlying decoder.
    ///
    /// The returned [`ByteChunk`] borrows the chunker's buffer; consume or
    /// copy [`ByteChunk::bytes`] before calling `next_chunk` again.
    ///
    /// # Errors
    /// This function returns an error if the underlying decoder errors. A
    /// decoder error is terminal: any records already buffered for the
    /// in-progress chunk are dropped, and subsequent calls return
    /// `Ok(None)` without touching the underlying decoder.
    pub fn next_chunk(&mut self) -> crate::Result<Option<ByteChunk<'_>>> {
        if self.done {
            return Ok(None);
        }
        self.buf.clear();
        let mut count: usize = 0;

        loop {
            if let Some(max) = self.max_records {
                if count >= max.get() {
                    break;
                }
            }
            let rec = match self.decoder.decode_record_ref() {
                Ok(Some(r)) => r,
                Ok(None) => {
                    self.done = true;
                    break;
                }
                Err(e) => {
                    self.done = true;
                    return Err(e);
                }
            };
            self.buf.extend_from_slice(rec.as_ref());
            count += 1;
            if self.buf.len() >= self.max_bytes.get() {
                break;
            }
        }

        Ok((count > 0).then(|| ByteChunk {
            bytes: &self.buf,
            count: count as u64,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        decode::DecodeRecordRef, rtype, test_utils::VecStream, MboMsg, Mbp1Msg, RecordHeader,
    };

    fn mbo(instrument_id: u32, publisher_id: u16, ts_event: u64) -> MboMsg {
        mbo_ts(instrument_id, publisher_id, ts_event, ts_event)
    }

    // `MboMsg::raw_index_ts` returns `ts_recv`, so the two timestamps are
    // split out for tests that care about the filter sort key.
    fn mbo_ts(instrument_id: u32, publisher_id: u16, ts_event: u64, ts_recv: u64) -> MboMsg {
        MboMsg {
            hd: RecordHeader::new::<MboMsg>(rtype::MBO, publisher_id, instrument_id, ts_event),
            ts_recv,
            ..Default::default()
        }
    }

    fn mbp1(instrument_id: u32, ts_event: u64, ts_recv: u64) -> Mbp1Msg {
        Mbp1Msg {
            hd: RecordHeader::new::<Mbp1Msg>(rtype::MBP_1, 1, instrument_id, ts_event),
            ts_recv,
            ..Default::default()
        }
    }

    fn nz(n: usize) -> NonZeroUsize {
        NonZeroUsize::new(n).unwrap()
    }

    fn drain_chunker<D: DecodeRecordRef>(
        chunker: &mut ByteChunker<D>,
    ) -> crate::Result<(Vec<u8>, u64)> {
        let mut all = Vec::new();
        let mut total = 0u64;
        while let Some(chunk) = chunker.next_chunk()? {
            all.extend_from_slice(chunk.bytes());
            total += chunk.count();
        }
        Ok((all, total))
    }

    fn drain_filter<D: DecodeRecordRef + private::LastRecord>(
        filter: &mut RecordFilter<D>,
    ) -> crate::Result<(Vec<u8>, u64)> {
        let mut out = Vec::new();
        let mut count = 0u64;
        while let Some(rec) = filter.decode_record_ref()? {
            out.extend_from_slice(rec.as_ref());
            count += 1;
        }
        Ok((out, count))
    }

    /// Test decoder that yields records until `fail_at`, then returns an error.
    struct FailingDecoder {
        records: Vec<MboMsg>,
        idx: usize,
        fail_at: usize,
    }

    impl FailingDecoder {
        fn new(records: Vec<MboMsg>, fail_at: usize) -> Self {
            Self {
                records,
                idx: 0,
                fail_at,
            }
        }
    }

    impl DecodeRecordRef for FailingDecoder {
        fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
            if self.idx == self.fail_at {
                return Err(crate::Error::decode("synthetic decoder error"));
            }
            let Some(rec) = self.records.get(self.idx) else {
                return Ok(None);
            };
            self.idx += 1;
            Ok(Some(RecordRef::from(rec)))
        }
    }

    impl private::LastRecord for FailingDecoder {
        fn last_record(&self) -> Option<RecordRef<'_>> {
            self.idx
                .checked_sub(1)
                .and_then(|i| self.records.get(i))
                .map(RecordRef::from)
        }
    }

    #[test]
    fn chunker_round_trip_verbatim() {
        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200), mbo(3, 20, 300)];
        let expected: Vec<u8> = recs
            .iter()
            .flat_map(|r| AsRef::<[u8]>::as_ref(r).to_vec())
            .collect();

        let mut chunker = ByteChunker::new(VecStream::new(recs));
        let (out, count) = drain_chunker(&mut chunker).unwrap();
        assert_eq!(count, 3);
        assert_eq!(out, expected);
    }

    #[test]
    fn chunker_max_records_splits_output() {
        let recs: Vec<_> = (0..10).map(|i| mbo(1, 10, i)).collect();
        let mut chunker = ByteChunker::new(VecStream::new(recs)).with_max_records(nz(4));
        let mut sizes = Vec::new();
        while let Some(chunk) = chunker.next_chunk().unwrap() {
            sizes.push(chunk.count());
        }
        assert_eq!(sizes, vec![4u64, 4, 2]);
    }

    #[test]
    fn chunker_max_records_one_yields_singleton_chunks() {
        let recs: Vec<_> = (0..3).map(|i| mbo(1, 10, i)).collect();
        let mut chunker = ByteChunker::new(VecStream::new(recs)).with_max_records(nz(1));
        let mut sizes = Vec::new();
        while let Some(chunk) = chunker.next_chunk().unwrap() {
            sizes.push(chunk.count());
        }
        assert_eq!(sizes, vec![1u64, 1, 1]);
    }

    // `max_bytes` is a soft ceiling: the chunk closes on the first record
    // that brings the buffer to or past the limit.
    #[test]
    fn chunker_max_bytes_splits_output() {
        let recs: Vec<_> = (0..5).map(|i| mbo(1, 10, i)).collect();
        let rec_size = std::mem::size_of::<MboMsg>();
        let limit = rec_size * 2;
        let mut chunker = ByteChunker::new(VecStream::new(recs)).with_max_bytes(nz(limit));
        let mut sizes = Vec::new();
        while let Some(chunk) = chunker.next_chunk().unwrap() {
            sizes.push(chunk.count());
            assert!(chunk.bytes().len() <= rec_size * 2);
        }
        assert_eq!(sizes, vec![2u64, 2, 1]);
    }

    // Both caps active: `max_bytes` trips before `max_records` would have,
    // so chunks close on bytes.
    #[test]
    fn chunker_max_bytes_trips_before_max_records() {
        let recs: Vec<_> = (0..6).map(|i| mbo(1, 10, i)).collect();
        let rec_size = std::mem::size_of::<MboMsg>();
        let mut chunker = ByteChunker::new(VecStream::new(recs))
            .with_max_records(nz(10))
            .with_max_bytes(nz(rec_size * 2));
        let mut sizes = Vec::new();
        while let Some(chunk) = chunker.next_chunk().unwrap() {
            sizes.push(chunk.count());
        }
        assert_eq!(sizes, vec![2u64, 2, 2]);
    }

    // A single record larger than `max_bytes` is still emitted (soft limit).
    #[test]
    fn chunker_max_bytes_below_one_record_still_emits() {
        let recs = vec![mbo(1, 10, 1), mbo(1, 10, 2)];
        let mut chunker = ByteChunker::new(VecStream::new(recs)).with_max_bytes(nz(1));
        let mut sizes = Vec::new();
        while let Some(chunk) = chunker.next_chunk().unwrap() {
            sizes.push(chunk.count());
        }
        assert_eq!(sizes, vec![1u64, 1]);
    }

    #[test]
    fn chunker_empty_input_yields_no_chunks() {
        let mut chunker = ByteChunker::new(VecStream::<MboMsg>::new(vec![]));
        let (out, count) = drain_chunker(&mut chunker).unwrap();
        assert_eq!(count, 0);
        assert!(out.is_empty());
    }

    #[test]
    fn chunker_idempotent_after_drain() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200)];
        let mut chunker = ByteChunker::new(VecStream::new(recs));
        while chunker.next_chunk().unwrap().is_some() {}
        assert!(chunker.next_chunk().unwrap().is_none());
        assert!(chunker.next_chunk().unwrap().is_none());
    }

    // A decoder error must propagate through `next_chunk` and be terminal:
    // subsequent calls return `Ok(None)` without touching the decoder.
    #[test]
    fn chunker_decoder_error_propagates_and_terminates() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200)];
        let mut chunker = ByteChunker::new(FailingDecoder::new(recs, 1));
        let err = chunker.next_chunk().unwrap_err();
        assert!(matches!(err, crate::Error::Decode(_)));
        assert!(chunker.next_chunk().unwrap().is_none());
        assert!(chunker.next_chunk().unwrap().is_none());
    }

    // `get_mut` exposes a mutable reference to the wrapped decoder.
    #[test]
    fn chunker_get_mut_reaches_inner_decoder() {
        let recs = vec![mbo(1, 10, 100)];
        let mut chunker = ByteChunker::new(VecStream::new(recs));
        let next = chunker.get_mut().decode_record_ref().unwrap().unwrap();
        assert_eq!(next.header().ts_event, 100);
    }

    // `into_inner` hands back the wrapped decoder after some chunks have
    // been pulled, positioned after whatever the chunker consumed.
    #[test]
    fn chunker_into_inner_returns_positioned_decoder() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200)];
        let mut chunker = ByteChunker::new(VecStream::new(recs)).with_max_records(nz(1));
        let chunk = chunker.next_chunk().unwrap().expect("chunk");
        assert_eq!(chunk.count(), 1);
        let mut inner = chunker.into_inner();
        let next = inner.decode_record_ref().unwrap().unwrap();
        assert_eq!(next.header().ts_event, 200);
    }

    #[test]
    fn chunker_default_max_bytes_is_4_mib() {
        assert_eq!(
            ByteChunker::<VecStream<MboMsg>>::DEFAULT_MAX_BYTES.get(),
            4 * 1024 * 1024,
        );
    }

    // With `max_records` unset, the only cap is `max_bytes`; 200 tiny records
    // fit in one chunk under a generous byte ceiling.
    #[test]
    fn chunker_default_has_no_record_cap() {
        let rec_size = std::mem::size_of::<MboMsg>();
        let recs: Vec<_> = (0..200).map(|i| mbo(1, 10, i)).collect();
        let mut chunker =
            ByteChunker::new(VecStream::new(recs)).with_max_bytes(nz(rec_size * 1000));
        let chunk = chunker.next_chunk().unwrap().expect("chunk");
        assert_eq!(chunk.count(), 200);
        assert!(chunker.next_chunk().unwrap().is_none());
    }

    // Iterates a V1 file with UpgradeToV3, confirms byte-for-byte equality
    // with a direct decode, then prepends V3 metadata and round-trips through
    // DbnDecoder.
    #[test]
    fn chunker_upgrades_v1_and_round_trips() {
        use crate::decode::{DbnDecoder, DynDecoder};
        use crate::enums::VersionUpgradePolicy;
        use std::io::Cursor;

        const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");
        let path = format!("{TEST_DATA_PATH}/test_data.mbo.v1.dbn.zst");

        let mut orig = DynDecoder::from_file(&path, VersionUpgradePolicy::UpgradeToV3).unwrap();
        let mut orig_bytes = Vec::new();
        let mut orig_count = 0u64;
        while let Some(rec) = orig.decode_record_ref().unwrap() {
            orig_bytes.extend_from_slice(rec.as_ref());
            orig_count += 1;
        }
        assert!(orig_count > 0);

        let chunker_decoder =
            DynDecoder::from_file(&path, VersionUpgradePolicy::UpgradeToV3).unwrap();
        let metadata = chunker_decoder.metadata().clone();
        let mut chunker = ByteChunker::new(chunker_decoder);
        let mut chunked = Vec::new();
        while let Some(chunk) = chunker.next_chunk().unwrap() {
            chunked.extend_from_slice(chunk.bytes);
        }
        assert_eq!(chunked, orig_bytes);

        let mut dbn_bytes = Vec::new();
        crate::encode::DbnMetadataEncoder::new(&mut dbn_bytes)
            .encode(&metadata)
            .unwrap();
        dbn_bytes.extend_from_slice(&chunked);

        let mut redec = DbnDecoder::new(Cursor::new(dbn_bytes)).unwrap();
        let mut redec_count = 0u64;
        while redec.decode_record_ref().unwrap().is_some() {
            redec_count += 1;
        }
        assert_eq!(redec_count, orig_count);
    }

    // `DbnMetadata` passes through to the wrapped decoder unchanged.
    #[test]
    fn chunker_metadata_pass_through() {
        use crate::decode::DynDecoder;
        use crate::enums::VersionUpgradePolicy;

        const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");
        let path = format!("{TEST_DATA_PATH}/test_data.mbo.v1.dbn.zst");
        let decoder = DynDecoder::from_file(&path, VersionUpgradePolicy::UpgradeToV3).unwrap();
        let expected = decoder.metadata().clone();
        let mut chunker = ByteChunker::new(decoder);
        assert_eq!(chunker.metadata(), &expected);
        chunker.metadata_mut().dataset = "mutated".to_owned();
        assert_eq!(chunker.get_ref().metadata().dataset, "mutated");
    }

    #[test]
    fn filter_passthrough_with_no_filters() {
        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200), mbo(3, 20, 300)];
        let expected: Vec<u8> = recs
            .iter()
            .flat_map(|r| AsRef::<[u8]>::as_ref(r).to_vec())
            .collect();
        let mut filter = RecordFilter::builder(VecStream::new(recs)).build().unwrap();
        let (out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 3);
        assert_eq!(out, expected);
    }

    #[test]
    fn filter_by_instrument_id() {
        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200), mbo(1, 10, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .build()
            .unwrap();
        let (out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(out.len(), 2 * std::mem::size_of::<MboMsg>());
        assert_eq!(filter.stats().dropped_by_instrument_id, 1);
    }

    #[test]
    fn filter_by_publisher_id() {
        let recs = vec![mbo(1, 10, 100), mbo(2, 20, 200), mbo(3, 10, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .publisher_ids([20u16])
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(filter.stats().dropped_by_publisher_id, 2);
    }

    #[test]
    fn filter_start_ts_skips_records_below() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200), mbo(1, 10, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .start_ts(200)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(filter.stats().emitted, 2);
        assert_eq!(filter.stats().dropped_by_time, 1);
    }

    // A filter built from a multi-element iterator must accept every listed id.
    #[test]
    fn filter_multi_value_filter() {
        let recs = vec![
            mbo(1, 10, 100),
            mbo(2, 20, 200),
            mbo(3, 30, 300),
            mbo(4, 40, 400),
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1, 3])
            .publisher_ids([10u16, 30, 40])
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(filter.stats().emitted, 2);
    }

    // Both `IntoIterator` shapes should compile: a slice-borrowing iterator
    // via `iter().copied()` and a `Vec` by value.
    #[test]
    fn filter_setters_accept_any_iterator() {
        let ids: &[u32] = &[1, 2];
        let recs = vec![mbo(1, 10, 1), mbo(3, 10, 2)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids(ids.iter().copied())
            .publisher_ids(vec![10u16])
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 1);
    }

    // `publisher_ids` accepts anything `Into<u16>`, including the `Publisher`
    // enum, so callers don't have to hand-convert to raw ids.
    #[test]
    fn filter_publisher_ids_accepts_publisher_enum() {
        use crate::Publisher;

        let glbx = u16::from(Publisher::GlbxMdp3Glbx);
        let xnas = u16::from(Publisher::XnasItchXnas);
        let recs = vec![mbo(1, glbx, 100), mbo(1, xnas, 200), mbo(1, glbx, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .publisher_ids([Publisher::GlbxMdp3Glbx])
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(filter.stats().dropped_by_publisher_id, 1);
    }

    // Each drop reason must be counted against its own counter, not a shared
    // bucket.
    #[test]
    fn filter_counters_separate_per_reason() {
        let recs = vec![
            mbo(1, 10, 100), // emit
            mbo(2, 10, 150), // drop (instrument)
            mbo(1, 20, 200), // drop (publisher)
            mbo(1, 10, 50),  // drop (time)
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .publisher_ids([10u16])
            .start_ts(100)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            filter.stats(),
            FilterStats {
                emitted: 1,
                dropped_by_time: 1,
                dropped_by_instrument_id: 1,
                dropped_by_publisher_id: 1,
            }
        );
    }

    #[test]
    fn filter_end_ts_terminates_early() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200), mbo(1, 10, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .end_ts(250)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(filter.stats().emitted, 2);
    }

    // `tripping_record()` stashes the record that tripped `end_ts`; the
    // inner decoder is positioned past it so a resume joins seamlessly.
    #[test]
    fn filter_tripping_record_stashed() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 300), mbo(1, 10, 400)];
        let expected_tripping: Vec<u8> = AsRef::<[u8]>::as_ref(&recs[1]).to_vec();
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .end_ts(250)
            .build()
            .unwrap();
        while filter.decode_record_ref().unwrap().is_some() {}
        let tripping_bytes = filter.tripping_record().map(AsRef::<[u8]>::as_ref);
        assert_eq!(tripping_bytes, Some(expected_tripping.as_slice()));
        let (mut inner, _) = filter.into_parts();
        let next = inner.decode_record_ref().unwrap().unwrap();
        assert_eq!(next.header().ts_event, 400);
    }

    // A tripping record that wouldn't have survived the non-time filters is
    // not stashed, so resuming through the raw decoder doesn't reintroduce a
    // record the caller had filtered out. Iteration still terminates.
    #[test]
    fn filter_drops_tripping_that_fails_header_filter() {
        let recs = vec![
            mbo(1, 10, 100),
            mbo(2, 10, 300), // trips end_ts but fails instrument filter
            mbo(1, 10, 400),
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .end_ts(250)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 1);
        assert!(filter.tripping_record().is_none());
    }

    #[test]
    fn filter_no_tripping_when_natural_end() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200)];
        let mut filter = RecordFilter::builder(VecStream::new(recs)).build().unwrap();
        while filter.decode_record_ref().unwrap().is_some() {}
        assert!(filter.tripping_record().is_none());
    }

    // A record at or past `end_ts` terminates iteration even when a filter
    // would otherwise have dropped it; drop counters must not count the
    // tripping record.
    #[test]
    fn filter_end_ts_terminates_before_filter_check() {
        let recs = vec![
            mbo(1, 10, 100),
            mbo(2, 10, 200), // instrument filter would drop; end_ts trips first
            mbo(1, 10, 300),
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .end_ts(200)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 1);
        assert_eq!(filter.stats().emitted, 1);
        assert_eq!(filter.stats().dropped_by_instrument_id, 0);
    }

    // `start_ts == end_ts` is allowed and yields an empty stream.
    #[test]
    fn filter_start_ts_equals_end_ts_emits_nothing() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200), mbo(1, 10, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .start_ts(200)
            .end_ts(200)
            .build()
            .unwrap();
        assert!(filter.decode_record_ref().unwrap().is_none());
        assert_eq!(filter.stats().emitted, 0);
    }

    // `start_ts > end_ts` is rejected at build time.
    #[test]
    fn filter_start_ts_greater_than_end_ts_errors() {
        let recs = vec![mbo(1, 10, 100)];
        let err = RecordFilter::builder(VecStream::new(recs))
            .start_ts(300)
            .end_ts(200)
            .build()
            .unwrap_err();
        assert!(
            matches!(err, crate::Error::BadArgument { ref param_name, .. } if param_name == "start_ts")
        );
    }

    #[test]
    fn filter_combined_filters_intersect() {
        let recs = vec![
            mbo(1, 10, 100), // keep
            mbo(2, 10, 150), // drop (instrument_id)
            mbo(1, 20, 200), // drop (publisher_id)
            mbo(1, 10, 250), // keep
            mbo(1, 10, 400), // drop (end_ts)
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .publisher_ids([10u16])
            .start_ts(0)
            .end_ts(300)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn filter_rejects_all() {
        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([999])
            .build()
            .unwrap();
        assert!(filter.decode_record_ref().unwrap().is_none());
        assert_eq!(filter.stats().emitted, 0);
        assert_eq!(filter.stats().dropped_by_instrument_id, 2);
    }

    // For MBO, `raw_index_ts` is `ts_recv` (not `ts_event`). `start_ts` /
    // `end_ts` must key off `ts_recv` so time filters match the sort order
    // of DBN files.
    #[test]
    fn filter_time_filters_use_ts_recv_for_mbo() {
        let recs = vec![
            mbo_ts(1, 10, 90, 100),
            mbo_ts(1, 10, 180, 200),
            mbo_ts(1, 10, 280, 300),
            mbo_ts(1, 10, 380, 400),
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .start_ts(150)
            .end_ts(350)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(filter.stats().emitted, 2);
        assert_eq!(filter.stats().dropped_by_time, 1); // ts_recv=100
    }

    #[test]
    fn filter_time_filters_use_ts_recv_for_mbp() {
        let recs = vec![
            mbp1(1, 90, 100),
            mbp1(1, 180, 200),
            mbp1(1, 280, 300),
            mbp1(1, 380, 400),
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .start_ts(150)
            .end_ts(350)
            .build()
            .unwrap();
        let (_out, count) = drain_filter(&mut filter).unwrap();
        assert_eq!(count, 2);
        assert_eq!(filter.stats().emitted, 2);
        assert_eq!(filter.stats().dropped_by_time, 1);
    }

    // Counters must survive across multiple `decode_record_ref` calls.
    #[test]
    fn filter_counters_monotonic() {
        let recs: Vec<_> = (0..10).map(|i| mbo(1 + (i % 2) as u32, 10, i)).collect();
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .build()
            .unwrap();
        let mut emitted = Vec::new();
        let mut dropped = Vec::new();
        while filter.decode_record_ref().unwrap().is_some() {
            let stats = filter.stats();
            emitted.push(stats.emitted);
            dropped.push(stats.dropped_by_instrument_id);
        }
        assert!(emitted.windows(2).all(|w| w[0] <= w[1]));
        assert!(dropped.windows(2).all(|w| w[0] <= w[1]));
        assert_eq!(filter.stats().emitted, 5);
        assert_eq!(filter.stats().dropped_by_instrument_id, 5);
    }

    // `tripping_record()` is observable before `into_parts` consumes the filter.
    #[test]
    fn filter_tripping_record_accessor_before_end() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 300), mbo(1, 10, 400)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .end_ts(250)
            .build()
            .unwrap();
        assert!(filter.tripping_record().is_none());
        while filter.decode_record_ref().unwrap().is_some() {}
        let trip = filter.tripping_record().expect("stashed");
        assert_eq!(trip.header().ts_event, 300);
    }

    // A decoder error must propagate through `decode_record_ref` and be
    // terminal; subsequent calls return `Ok(None)`. Drops preceding the
    // error are counted normally.
    #[test]
    fn filter_decoder_error_propagates_and_terminates() {
        let recs = vec![mbo(2, 10, 100), mbo(1, 10, 200)];
        let mut filter = RecordFilter::builder(FailingDecoder::new(recs, 1))
            .instrument_ids([1])
            .build()
            .unwrap();
        let err = filter.decode_record_ref().unwrap_err();
        assert!(matches!(err, crate::Error::Decode(_)));
        assert!(filter.decode_record_ref().unwrap().is_none());
        assert!(filter.decode_record_ref().unwrap().is_none());
        assert_eq!(filter.stats().dropped_by_instrument_id, 1);
    }

    // Post-drain calls must return `Ok(None)` and leave counters unchanged.
    #[test]
    fn filter_idempotent_after_drain() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200)];
        let mut filter = RecordFilter::builder(VecStream::new(recs)).build().unwrap();
        while filter.decode_record_ref().unwrap().is_some() {}
        let emitted = filter.stats().emitted;
        assert!(filter.decode_record_ref().unwrap().is_none());
        assert!(filter.decode_record_ref().unwrap().is_none());
        assert_eq!(filter.stats().emitted, emitted);
    }

    // When a record would fail multiple filters, classification halts at
    // the first matching reason (end_ts > start_ts > instrument > publisher),
    // so only one counter increments per dropped record.
    #[test]
    fn filter_counts_first_matching_reason() {
        let recs = vec![
            // Fails both instrument and publisher; only instrument counts.
            mbo(2, 20, 100),
            // Fails both start_ts and instrument; only start_ts counts.
            mbo(2, 10, 50),
        ];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .publisher_ids([10u16])
            .start_ts(100)
            .build()
            .unwrap();
        while filter.decode_record_ref().unwrap().is_some() {}
        assert_eq!(
            filter.stats(),
            FilterStats {
                emitted: 0,
                dropped_by_time: 1,
                dropped_by_instrument_id: 1,
                dropped_by_publisher_id: 0,
            }
        );
    }

    // A `RecordFilter` wrapped inside a `ByteChunker` applies filters before
    // the chunker buffers bytes, matching the documented pipeline shape.
    #[test]
    fn filter_into_chunker_composes() {
        let recs = vec![
            mbo(1, 10, 100),
            mbo(2, 10, 200), // dropped by instrument filter
            mbo(1, 10, 300),
        ];
        let filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .build()
            .unwrap();
        let mut chunker = ByteChunker::new(filter);
        let (_out, total) = drain_chunker(&mut chunker).unwrap();
        assert_eq!(total, 2);
    }

    // `DbnMetadata` propagates through both wrappers: a `ByteChunker` over
    // a `RecordFilter` exposes the inner decoder's metadata.
    #[test]
    fn filter_and_chunker_metadata_pass_through() {
        use crate::decode::DynDecoder;
        use crate::enums::VersionUpgradePolicy;

        const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/data");
        let path = format!("{TEST_DATA_PATH}/test_data.mbo.v1.dbn.zst");
        let decoder = DynDecoder::from_file(&path, VersionUpgradePolicy::UpgradeToV3).unwrap();
        let expected = decoder.metadata().clone();
        let filter = RecordFilter::builder(decoder).build().unwrap();
        let chunker = ByteChunker::new(filter);
        assert_eq!(chunker.metadata(), &expected);
    }

    // Filter + chunker: unwrap twice to recover the raw decoder and the
    // stashed tripping record for a seamless resume.
    #[test]
    fn filter_into_chunker_end_ts_resume() {
        let recs = vec![
            mbo(1, 10, 100),
            mbo(1, 10, 200),
            mbo(1, 10, 300), // trips end_ts
            mbo(1, 10, 400),
        ];
        let expected_tripping: Vec<u8> = AsRef::<[u8]>::as_ref(&recs[2]).to_vec();
        let filter = RecordFilter::builder(VecStream::new(recs))
            .end_ts(250)
            .build()
            .unwrap();
        let mut chunker = ByteChunker::new(filter);
        let (_, total) = drain_chunker(&mut chunker).unwrap();
        assert_eq!(total, 2);

        let filter = chunker.into_inner();
        let (mut decoder, trip) = filter.into_parts();
        let trip = trip.expect("stashed");
        assert_eq!(AsRef::<[u8]>::as_ref(&trip), expected_tripping.as_slice());
        let next = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(next.header().ts_event, 400);
    }

    // `into_parts` hands back both the decoder and the stashed tripping
    // record in one move, so resume code doesn't have to clone the record
    // out of a borrow before consuming the filter.
    #[test]
    fn filter_into_parts_returns_decoder_and_tripping() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 300), mbo(1, 10, 400)];
        let expected_tripping: Vec<u8> = AsRef::<[u8]>::as_ref(&recs[1]).to_vec();
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .end_ts(250)
            .build()
            .unwrap();
        while filter.decode_record_ref().unwrap().is_some() {}
        let (mut decoder, trip) = filter.into_parts();
        let trip = trip.expect("stashed");
        assert_eq!(AsRef::<[u8]>::as_ref(&trip), expected_tripping.as_slice());
        let next = decoder.decode_record_ref().unwrap().unwrap();
        assert_eq!(next.header().ts_event, 400);
    }

    // When iteration ends naturally, `into_parts` yields `None` for the
    // tripping record.
    #[test]
    fn filter_into_parts_no_tripping_on_natural_end() {
        let recs = vec![mbo(1, 10, 100), mbo(1, 10, 200)];
        let mut filter = RecordFilter::builder(VecStream::new(recs)).build().unwrap();
        while filter.decode_record_ref().unwrap().is_some() {}
        let (_decoder, trip) = filter.into_parts();
        assert!(trip.is_none());
    }

    // `RecordFilter` implements `DecodeRecord`, so it composes anywhere the
    // rest of the crate's decoders do: typed single-record decoding, the
    // `decode_records` vector helper, and a streaming iterator.
    #[test]
    fn filter_decode_record_returns_typed_record() {
        use crate::decode::DecodeRecord;

        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200), mbo(1, 10, 300)];
        let mut filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .build()
            .unwrap();
        let first = filter.decode_record::<MboMsg>().unwrap().unwrap();
        assert_eq!(first.ts_recv, 100);
        let second = filter.decode_record::<MboMsg>().unwrap().unwrap();
        assert_eq!(second.ts_recv, 300);
        assert!(filter.decode_record::<MboMsg>().unwrap().is_none());
    }

    #[test]
    fn filter_decode_records_collects_filtered() {
        use crate::decode::DecodeRecord;

        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200), mbo(1, 10, 300)];
        let filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .build()
            .unwrap();
        let collected: Vec<MboMsg> = filter.decode_records().unwrap();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].ts_recv, 100);
        assert_eq!(collected[1].ts_recv, 300);
    }

    #[test]
    fn filter_decode_stream_yields_filtered() {
        use crate::decode::DecodeStream;
        use fallible_streaming_iterator::FallibleStreamingIterator;

        let recs = vec![mbo(1, 10, 100), mbo(2, 10, 200), mbo(1, 10, 300)];
        let filter = RecordFilter::builder(VecStream::new(recs))
            .instrument_ids([1])
            .build()
            .unwrap();
        let mut stream = filter.decode_stream::<MboMsg>();
        let mut seen = Vec::new();
        while let Some(rec) = stream.next().unwrap() {
            seen.push(rec.ts_recv);
        }
        assert_eq!(seen, vec![100, 300]);
    }
}
