use std::{cmp::Reverse, collections::BinaryHeap};

use crate::{Error, HasRType, Metadata, Record, RecordRef};

use super::{private, DbnMetadata, DecodeRecord, DecodeRecordRef, DecodeStream, StreamIterDecoder};

/// Merges the DBN decoding streams from one or more other decoders. Both metadata and
/// the record streams are merged.
pub struct Decoder<D> {
    metadata: Metadata,
    decoder: RecordDecoder<D>,
}

impl<D> Decoder<D>
where
    D: DbnMetadata + DecodeRecordRef,
{
    /// Creates a new merge decoder from the given `decoders`. Both the DBN metadata and
    /// the records will be merged.
    ///
    /// # Errors
    /// This function returns an error if `decoders` is empty. Errors can also result from
    /// failing to merge the DBN metadata. It will also return an error if one of the
    /// inner decoders returns an error while decoding the first record. A decoder
    /// returning `Ok(None)` does not result in a failure.
    pub fn new(decoders: Vec<D>) -> crate::Result<Self> {
        let Some((first, rest)) = decoders.split_first() else {
            return Err(Error::BadArgument {
                param_name: "decoders".to_owned(),
                desc: "none provided".to_owned(),
            });
        };
        let metadata = first
            .metadata()
            .clone()
            .merge(rest.iter().map(|d| d.metadata().clone()))?;
        Ok(Self {
            metadata,
            decoder: RecordDecoder::new(decoders)?,
        })
    }
}

impl<D> DbnMetadata for Decoder<D> {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl<D> DecodeRecordRef for Decoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>> {
        self.decoder.decode_record_ref()
    }
}

impl<D> DecodeRecord for Decoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        self.decoder.decode_record()
    }
}

impl<D> private::LastRecord for Decoder<D>
where
    D: private::LastRecord,
{
    fn last_record(&self) -> Option<RecordRef> {
        self.decoder.last_record()
    }
}

impl<D> DecodeStream for Decoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_stream<T: HasRType>(self) -> StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        StreamIterDecoder::new(self)
    }
}

/// Merges the record decoding streams from one or more other decoders, performing a
/// k-merge based on [`Record::index_ts()`].
pub struct RecordDecoder<D> {
    /// Should never change size because [`min_heap`] holds indices to this `Vec`.
    decoders: Vec<D>,
    /// heap for kmerge
    min_heap: BinaryHeap<Reverse<StreamHead>>,
    is_first: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StreamHead {
    raw_index_ts: u64,
    decoder_idx: usize,
}

impl<D> RecordDecoder<D>
where
    D: DecodeRecordRef,
{
    /// Creates a new record-stream merging decoder.
    ///
    /// # Errors
    /// This function returns an error if `decoders` is empty. It will also return an
    /// error if one of the inner decoders returns an error while decoding the first
    /// record. A decoder returning `Ok(None)` does not result in a failure.
    pub fn new(mut decoders: Vec<D>) -> crate::Result<Self> {
        if decoders.is_empty() {
            return Err(Error::BadArgument {
                param_name: "decoders".to_owned(),
                desc: "none provided".to_owned(),
            });
        };
        let mut min_heap = BinaryHeap::new();
        // Populate heap for first time or all streams fully processed
        for (decoder_idx, decoder) in decoders.iter_mut().enumerate() {
            if let Some(rec) = decoder.decode_record_ref()? {
                min_heap.push(Reverse(StreamHead {
                    raw_index_ts: rec.raw_index_ts(),
                    decoder_idx,
                }));
            };
        }
        Ok(Self {
            decoders,
            min_heap,
            is_first: true,
        })
    }
}

impl<D> RecordDecoder<D> {
    fn peek_decoder_idx(&self) -> Option<usize> {
        self.min_heap
            .peek()
            .map(|Reverse(StreamHead { decoder_idx, .. })| *decoder_idx)
    }
}

impl<D> DecodeRecordRef for RecordDecoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef>> {
        if self.is_first {
            self.is_first = false;
        } else {
            // Pop last record
            let Some(Reverse(StreamHead {
                raw_index_ts: _,
                decoder_idx,
            })) = self.min_heap.pop()
            else {
                return Ok(None);
            };
            if let Some(rec) = self.decoders[decoder_idx].decode_record_ref()? {
                self.min_heap.push(Reverse(StreamHead {
                    raw_index_ts: rec.raw_index_ts(),
                    decoder_idx,
                }));
            }
        }
        let Some(decoder_idx) = self.peek_decoder_idx() else {
            return Ok(None);
        };
        Ok(self.decoders[decoder_idx].last_record())
    }
}

impl<D> DecodeRecord for RecordDecoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_record<T: HasRType>(&mut self) -> crate::Result<Option<&T>> {
        super::decode_record_from_ref(self.decode_record_ref()?)
    }
}

impl<D> private::LastRecord for RecordDecoder<D>
where
    D: private::LastRecord,
{
    fn last_record(&self) -> Option<RecordRef> {
        let Some(decoder_idx) = self.peek_decoder_idx() else {
            return self.decoders[0].last_record();
        };
        self.decoders[decoder_idx].last_record()
    }
}

impl<D> DecodeStream for RecordDecoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_stream<T: HasRType>(self) -> super::StreamIterDecoder<Self, T>
    where
        Self: Sized,
    {
        StreamIterDecoder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use fallible_streaming_iterator::FallibleStreamingIterator;

    use crate::{rtype, test_utils::VecStream, Mbp1Msg, Record, RecordHeader};

    use super::*;

    fn new_mbp1(ts_recv: u64) -> Mbp1Msg {
        Mbp1Msg {
            hd: RecordHeader::new::<Mbp1Msg>(rtype::MBP_1, 0, 0, 0),
            ts_recv,
            ..Default::default()
        }
    }

    #[test]
    fn stream_merging() {
        let target = RecordDecoder::new(vec![
            VecStream::new(vec![new_mbp1(10), new_mbp1(100), new_mbp1(1000)]),
            VecStream::new(vec![
                new_mbp1(11),
                new_mbp1(12),
                new_mbp1(13),
                new_mbp1(14),
                new_mbp1(15),
                new_mbp1(101),
                new_mbp1(102),
                new_mbp1(103),
                new_mbp1(104),
                new_mbp1(105),
            ]),
            VecStream::new(vec![
                new_mbp1(50),
                new_mbp1(105),
                new_mbp1(500),
                new_mbp1(5000),
            ]),
        ])
        .unwrap()
        .decode_stream::<Mbp1Msg>();
        let mut timestamps = Vec::new();
        let mut iter = target.map(|rec| rec.raw_index_ts());
        while let Some(ts) = iter.next().unwrap() {
            timestamps.push(*ts);
        }
        assert_eq!(
            timestamps,
            vec![10, 11, 12, 13, 14, 15, 50, 100, 101, 102, 103, 104, 105, 105, 500, 1000, 5000]
        );
        // extra advances should do nothing
        assert!(iter.next().unwrap().is_none());
        assert!(iter.next().unwrap().is_none());
    }

    #[test]
    fn stream_merging_w_empty() {
        let target = RecordDecoder::new(vec![
            VecStream::new(Vec::new()),
            VecStream::new(vec![new_mbp1(10), new_mbp1(100)]),
            VecStream::new(Vec::new()),
            VecStream::new(vec![
                new_mbp1(11),
                new_mbp1(12),
                new_mbp1(13),
                new_mbp1(14),
                new_mbp1(15),
            ]),
            VecStream::new(vec![new_mbp1(1), new_mbp1(50)]),
            VecStream::new(Vec::new()),
        ])
        .unwrap()
        .decode_stream::<Mbp1Msg>();
        let mut timestamps = Vec::new();
        let mut iter = target.map(|rec| rec.raw_index_ts());
        while let Some(ts) = iter.next().unwrap() {
            timestamps.push(*ts);
        }
        assert_eq!(timestamps, vec![1, 10, 11, 12, 13, 14, 15, 50, 100]);
    }
}
