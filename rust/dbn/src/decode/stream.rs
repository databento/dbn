use std::marker::PhantomData;

use fallible_streaming_iterator::FallibleStreamingIterator;

use super::{DecodeRecord, DecodeStream};
use crate::{record::transmute_record_bytes, Error, HasRType, Result};

/// A consuming iterator wrapping a [`DecodeRecord`]. Lazily decodes the contents of the
/// file or other input stream.
///
/// Implements [`FallibleStreamingIterator`].
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
            _marker: PhantomData,
        }
    }
}

impl<D, T> FallibleStreamingIterator for StreamIterDecoder<D, T>
where
    D: DecodeStream,
    T: HasRType,
{
    type Error = Error;
    type Item = T;

    fn advance(&mut self) -> Result<()> {
        if let Some(i) = self.i.as_mut() {
            match self.decoder.decode_record::<T>() {
                Ok(Some(_)) => {
                    *i += 1;
                    Ok(())
                }
                Ok(None) => {
                    // set error state sentinel
                    self.i = None;
                    Ok(())
                }
                Err(err) => {
                    // set error state sentinel
                    self.i = None;
                    Err(err)
                }
            }
        } else {
            Ok(())
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
