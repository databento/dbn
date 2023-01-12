use std::marker::PhantomData;

use streaming_iterator::StreamingIterator;

use super::DecodeDbn;
use crate::record::{transmute_record_bytes, HasRType};

/// A consuming iterator wrapping a [`DecodeDbn`]. Lazily decodes the contents of the file
/// or other input stream.
///
/// Implements [`streaming_iterator::StreamingIterator`].
pub struct StreamIterDecoder<D, T>
where
    D: DecodeDbn,
    T: HasRType,
{
    /// The underlying decoder implementation.
    decoder: D,
    /// Number of element sthat have been decoded. Used for [`Iterator::size_hint()`].
    i: usize,
    /// Required to associate this type with a specific record type `T`.
    _marker: PhantomData<T>,
}

impl<D, T> StreamIterDecoder<D, T>
where
    D: DecodeDbn,
    T: HasRType,
{
    pub(crate) fn new(decoder: D) -> Self {
        Self {
            decoder,
            i: 0,
            _marker: PhantomData,
        }
    }
}

impl<D, T> StreamingIterator for StreamIterDecoder<D, T>
where
    D: DecodeDbn,
    T: HasRType,
{
    type Item = T;

    fn advance(&mut self) {
        if self.decoder.decode_record::<T>().is_none() {
            // warn!("Failed to read from DBZ decoder: {e:?}");
            // set error state sentinel
            self.i = self.decoder.metadata().record_count as usize + 1;
        }
        self.i += 1;
    }

    fn get(&self) -> Option<&Self::Item> {
        // TODO(cg): improve error handling
        if self.i > self.decoder.metadata().record_count as usize {
            return None;
        }
        // Safety: `buffer` is specifically sized to `T`
        unsafe { transmute_record_bytes(self.decoder.buffer_slice()) }
    }

    /// Returns the lower bound and upper bounds of remaining length of iterator.
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.decoder.metadata().record_count == u64::MAX {
            (0, None)
        } else {
            let remaining = self.decoder.metadata().record_count as usize - self.i;
            // assumes `record_count` is accurate. If it is not, the program won't crash but
            // performance will be suboptimal
            (remaining, Some(remaining))
        }
    }
}
