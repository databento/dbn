use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    decode::{private::LastRecord, DecodeRecordRef},
    Error, HasRType, RecordRef,
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
    T: HasRType,
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
    T: HasRType + AsRef<[u8]>,
{
    fn last_record(&self) -> Option<RecordRef<'_>> {
        if self.vec.is_empty() {
            None
        } else {
            Some(RecordRef::from(self.vec.get(self.idx as usize).unwrap()))
        }
    }
}
