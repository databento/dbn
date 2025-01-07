use fallible_streaming_iterator::FallibleStreamingIterator;

use crate::{
    decode::{private::BufferSlice, DecodeRecordRef},
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
    fn decode_record_ref(&mut self) -> crate::Result<Option<crate::RecordRef>> {
        self.idx += 1;
        let Some(rec) = self.vec.get(self.idx as usize) else {
            return Ok(None);
        };
        Ok(Some(RecordRef::from(rec)))
    }
}

impl<T> BufferSlice for VecStream<T>
where
    T: HasRType + AsRef<[u8]>,
{
    fn buffer_slice(&self) -> &[u8] {
        self.vec
            .get(self.idx as usize)
            .map(|r| r.as_ref())
            .unwrap_or_default()
    }

    fn compat_buffer_slice(&self) -> &[u8] {
        &[]
    }

    fn record_ref(&self) -> RecordRef {
        RecordRef::from(self.vec.get(self.idx as usize).unwrap())
    }
}
