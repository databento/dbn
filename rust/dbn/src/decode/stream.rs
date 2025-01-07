use std::marker::PhantomData;

use fallible_streaming_iterator::FallibleStreamingIterator;

use super::{DbnMetadata, DecodeStream};
use crate::{Error, HasRType, Result};

/// A consuming iterator wrapping a [`DecodeRecord`](super::DecodeRecord). Lazily
/// decodes the contents of the file or other input stream.
///
/// Implements [`FallibleStreamingIterator`].
pub struct StreamIterDecoder<D, T> {
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
            // SAFETY: Validated record type in `advance` with call to `decode_record`.
            Some(unsafe { self.decoder.record_ref().get_unchecked() })
        } else {
            None
        }
    }
}

impl<D, T> DbnMetadata for StreamIterDecoder<D, T>
where
    D: DbnMetadata,
{
    fn metadata(&self) -> &crate::Metadata {
        self.decoder.metadata()
    }

    fn metadata_mut(&mut self) -> &mut crate::Metadata {
        self.decoder.metadata_mut()
    }
}
