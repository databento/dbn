use std::marker::PhantomData;

use streaming_iterator::StreamingIterator;

use super::{DecodeRecord, DecodeStream};
use crate::record::{transmute_record_bytes, HasRType};

/// A consuming iterator wrapping a [`DecodeRecord`]. Lazily decodes the contents of the
/// file or other input stream.
///
/// Implements [`streaming_iterator::StreamingIterator`].
pub struct StreamIterDecoder<D, T>
where
    D: DecodeRecord,
    T: HasRType,
{
    /// The underlying decoder implementation.
    decoder: D,
    /// Number of element sthat have been decoded. Used for [`Iterator::size_hint()`].
    /// `None` indicates the end of the stream has been reached.
    i: Option<usize>,
    /// Last error encountering when decoding.
    last_err: Option<crate::Error>,
    /// Required to associate this type with a specific record type `T`.
    _marker: PhantomData<T>,
}

impl<D, T> StreamIterDecoder<D, T>
where
    D: DecodeRecord,
    T: HasRType,
{
    /// Creates a new streaming decoder using the given `decoder`.
    pub fn new(decoder: D) -> Self {
        Self {
            decoder,
            i: Some(0),
            last_err: None,
            _marker: PhantomData,
        }
    }

    /// Last error encountering when decoding.
    pub fn last_err(&self) -> Option<&crate::Error> {
        self.last_err.as_ref()
    }
}

impl<D, T> StreamingIterator for StreamIterDecoder<D, T>
where
    D: DecodeStream,
    T: HasRType,
{
    type Item = T;

    fn advance(&mut self) {
        if let Some(i) = self.i.as_mut() {
            match self.decoder.decode_record::<T>() {
                Err(err) => {
                    self.last_err = Some(err);
                    // set error state sentinel
                    self.i = None;
                }
                Ok(None) => {
                    // set error state sentinel
                    self.i = None;
                }
                Ok(Some(_)) => {
                    *i += 1;
                }
            }
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        if self.i.is_some() {
            // Safety: `buffer` is specifically sized to `T` and `i` has been
            // checked to see that the end of the stream hasn't been reached
            unsafe { transmute_record_bytes(self.decoder.buffer_slice()) }
        } else {
            None
        }
    }
}
