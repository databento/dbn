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
        let hints = decoders.iter().map(|d| d.metadata().start).collect();
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
            decoder: RecordDecoder::with_hints(decoders, hints)?,
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
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
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
    fn last_record(&self) -> Option<RecordRef<'_>> {
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
    index_ts: IndexTs,
    decoder_idx: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IndexTs {
    Real(u64),
    Hint(u64),
}

impl IndexTs {
    fn ts(&self) -> u64 {
        match self {
            IndexTs::Real(t) => *t,
            IndexTs::Hint(t) => *t,
        }
    }
}

impl PartialOrd for IndexTs {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IndexTs {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ts().cmp(&other.ts())
    }
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
        let mut min_heap = BinaryHeap::with_capacity(decoders.len());
        // Populate heap for first time or all streams fully processed
        for (decoder_idx, decoder) in decoders.iter_mut().enumerate() {
            if let Some(rec) = decoder.decode_record_ref()? {
                min_heap.push(Reverse(StreamHead {
                    index_ts: IndexTs::Real(rec.raw_index_ts()),
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

    /// Creates a new record-stream merging decoder with a hint for the start time
    /// for each decoder. This can assist the merger to avoid reading from a decoder
    /// before necessary.
    ///
    /// The hint timestamp must be <= raw_index_ts() of the first record in the file.
    /// [`Metadata::start`] is an example source of for hint.
    ///
    /// # Errors
    /// This function returns an error if `decoders` is empty or `decoders` and
    /// `start_ts_hints` are of different lengths. It will also return an error if
    /// one of the inner decoders returns an error while decoding the first record. A
    /// decoder returning `Ok(None)` does not result in a failure.
    pub fn with_hints(decoders: Vec<D>, start_ts_hints: Vec<u64>) -> crate::Result<Self> {
        if decoders.is_empty() {
            return Err(Error::BadArgument {
                param_name: "decoders".to_owned(),
                desc: "none provided".to_owned(),
            });
        };
        if decoders.len() != start_ts_hints.len() {
            return Err(Error::BadArgument {
                param_name: "hints".to_owned(),
                desc: "must have the same length as `decoders`".to_owned(),
            });
        }
        let min_heap = start_ts_hints
            .into_iter()
            .enumerate()
            .map(|(decoder_idx, hint)| {
                Reverse(StreamHead {
                    index_ts: IndexTs::Hint(hint),
                    decoder_idx,
                })
            })
            .collect();
        Ok(Self {
            decoders,
            min_heap,
            is_first: true,
        })
    }
}

impl<D> RecordDecoder<D> {
    // does not handle hints. Should only be called after `decode_record_ref`
    fn peek_decoder_idx(&self) -> Option<usize> {
        self.min_heap
            .peek()
            .map(|Reverse(StreamHead { decoder_idx, .. })| *decoder_idx)
    }
}

impl<D> RecordDecoder<D>
where
    D: DecodeRecordRef,
{
    // handles hints
    fn next_decoder_idx(&mut self) -> crate::Result<Option<usize>> {
        loop {
            let Some(Reverse(StreamHead {
                index_ts,
                decoder_idx,
            })) = self.min_heap.peek().cloned()
            else {
                return Ok(None);
            };
            match index_ts {
                IndexTs::Real(_) => return Ok(Some(decoder_idx)),
                IndexTs::Hint(_) => {
                    self.min_heap.pop();
                    if let Some(rec) = self.decoders[decoder_idx].decode_record_ref()? {
                        self.min_heap.push(Reverse(StreamHead {
                            index_ts: IndexTs::Real(rec.raw_index_ts()),
                            decoder_idx,
                        }));
                    }
                }
            }
        }
    }
}

impl<D> DecodeRecordRef for RecordDecoder<D>
where
    D: private::LastRecord + DecodeRecordRef,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<RecordRef<'_>>> {
        if self.is_first {
            self.is_first = false;
        } else {
            // Pop last record
            let Some(Reverse(StreamHead {
                index_ts: _,
                decoder_idx,
            })) = self.min_heap.pop()
            else {
                return Ok(None);
            };
            if let Some(rec) = self.decoders[decoder_idx].decode_record_ref()? {
                self.min_heap.push(Reverse(StreamHead {
                    index_ts: IndexTs::Real(rec.raw_index_ts()),
                    decoder_idx,
                }));
            }
        }
        let Some(decoder_idx) = self.next_decoder_idx()? else {
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
        self.decode_record_ref().and_then(|rec| {
            if let Some(rec) = rec {
                rec.try_get().map(Some)
            } else {
                Ok(None)
            }
        })
    }
}

impl<D> private::LastRecord for RecordDecoder<D>
where
    D: private::LastRecord,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
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
    use rstest::*;

    use crate::{rtype, test_utils::VecStream, Mbp1Msg, Record, RecordHeader};

    use super::*;

    fn new_mbp1(ts_recv: u64) -> Mbp1Msg {
        Mbp1Msg {
            hd: RecordHeader::new::<Mbp1Msg>(rtype::MBP_1, 0, 0, 0),
            ts_recv,
            ..Default::default()
        }
    }

    #[rstest]
    fn stream_merging(#[values(None, Some(vec![5, 1, 50]))] hints: Option<Vec<u64>>) {
        let decoders = vec![
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
        ];
        let target = if let Some(hints) = hints {
            RecordDecoder::with_hints(decoders, hints)
        } else {
            RecordDecoder::new(decoders)
        }
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

    #[rstest]
    fn stream_merging_w_empty(
        #[values(None, Some(vec![0, 10, 11, 1, 1, 50]))] hints: Option<Vec<u64>>,
    ) {
        let decoders = vec![
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
        ];
        let target = if let Some(hints) = hints {
            RecordDecoder::with_hints(decoders, hints)
        } else {
            RecordDecoder::new(decoders)
        }
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
