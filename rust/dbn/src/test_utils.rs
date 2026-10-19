use std::os::raw::c_char;

use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    decode::{private::LastRecord, DecodeRecordRef},
    v3, v4, Error, FlagSet, RecordRef,
};

/// A testing shim to get a streaming iterator from a [`Vec`].
pub struct VecStream<T> {
    vec: Vec<T>,
    idx: isize,
}

impl<T> VecStream<T> {
    pub fn new(vec: Vec<T>) -> Self {
        // initialize at -1 because `advance()` is always called before
        // `get()`.
        Self { vec, idx: -1 }
    }
}

impl<T> FallibleStreamingIterator for VecStream<T> {
    type Item = T;
    type Error = Error;

    fn advance(&mut self) -> Result<(), Error> {
        self.idx += 1;
        Ok(())
    }

    fn get(&self) -> Option<&Self::Item> {
        self.vec.get(self.idx as usize)
    }
}

impl<T> DecodeRecordRef for VecStream<T>
where
    T: v3::HasRType,
{
    fn decode_record_ref(&mut self) -> crate::Result<Option<crate::RecordRef<'_>>> {
        self.idx += 1;
        let Some(rec) = self.vec.get(self.idx as usize) else {
            return Ok(None);
        };
        Ok(Some(RecordRef::from(rec)))
    }
}

impl<T> LastRecord for VecStream<T>
where
    T: v3::HasRType + AsRef<[u8]>,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
        if self.vec.is_empty() {
            None
        } else {
            Some(RecordRef::from(self.vec.get(self.idx as usize).unwrap()))
        }
    }
}

pub fn mbo_v4() -> v4::MboMsg {
    v4::MboMsg {
        hd: v4::RecordHeader::new::<v4::MboMsg>(v4::rtype::MBO, 1, 5482, 1_697_240_529_000_000_000),
        order_id: 16,
        price: 5500,
        size: 3,
        flags: FlagSet::empty(),
        channel_id: 14,
        action: b'A' as c_char,
        side: b'B' as c_char,
        ts_recv: 1_697_240_529_000_000_100,
        ts_in_delta: 22_000,
        sequence: 1_002_375,
    }
}

pub fn mbo_v3() -> v3::MboMsg {
    let v4 = mbo_v4();
    v3::MboMsg {
        hd: v3::RecordHeader::new::<v3::MboMsg>(
            crate::rtype::MBO,
            v4.hd.publisher_id,
            u32::try_from(v4.hd.instrument_id).unwrap(),
            v4.hd.ts_event,
        ),
        order_id: v4.order_id,
        price: v4.price,
        size: v4.size,
        flags: v4.flags,
        channel_id: v4.channel_id,
        action: v4.action,
        side: v4.side,
        ts_recv: v4.ts_recv,
        ts_in_delta: v4.ts_in_delta,
        sequence: v4.sequence,
    }
}

pub fn trade_v4() -> v4::TradeMsg {
    v4::TradeMsg {
        hd: v4::RecordHeader::new::<v4::TradeMsg>(
            v4::rtype::MBP_0,
            1,
            5482,
            1_697_240_529_000_000_000,
        ),
        price: 5500,
        size: 7,
        action: b'T' as c_char,
        side: b'B' as c_char,
        flags: FlagSet::empty(),
        _reserved: 0,
        ts_recv: 1_697_240_529_000_000_100,
        ts_in_delta: 9,
        sequence: 88,
    }
}

pub fn trade_v3() -> v3::TradeMsg {
    let v4 = trade_v4();
    v3::TradeMsg {
        hd: v3::RecordHeader::new::<v3::TradeMsg>(
            crate::rtype::MBP_0,
            v4.hd.publisher_id,
            u32::try_from(v4.hd.instrument_id).unwrap(),
            v4.hd.ts_event,
        ),
        price: v4.price,
        size: v4.size,
        action: v4.action,
        side: v4.side,
        flags: v4.flags,
        depth: 0,
        ts_recv: v4.ts_recv,
        ts_in_delta: v4.ts_in_delta,
        sequence: v4.sequence,
    }
}
