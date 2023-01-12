//! NOTE: This module is somewhat experimental, as using it would require a lot of
//! duplicate functionality on both the encoding and decoding sides.
use std::{marker::PhantomData, mem, ptr::NonNull};

use crate::record::{HasRType, RecordHeader};

/// A wrapper around a non-owning reference to a DBN record. This wrapper allows for mixing of
/// record types and schemas, and runtime record polymorphism.
#[derive(Clone, Debug)]
pub struct RecordRef<'a> {
    ptr: NonNull<RecordHeader>,
    _marker: PhantomData<&'a RecordHeader>,
}

impl<'a> RecordRef<'a> {
    /// Constructs a new reference to the DBN record in `buffer`.
    ///
    /// # Safety
    /// `buffer` should begin with a [`RecordHeader`] and contain a type implementing
    /// [`HasRType`].
    pub unsafe fn new(buffer: &'a mut [u8]) -> Self {
        debug_assert!(buffer.len() >= mem::size_of::<RecordHeader>());
        let ptr = NonNull::new_unchecked(buffer.as_mut_ptr().cast::<RecordHeader>());
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Returns a reference to the [`RecordHeader`] of the referenced record.
    pub fn header(&self) -> &RecordHeader {
        // Safety: assumes `ptr` passes to a `RecordHeader`.
        unsafe { self.ptr.as_ref() }
    }

    /// Returns `true` if the object points to a record of type `T`.
    pub fn has<T: HasRType>(&self) -> bool {
        T::has_rtype(self.header().rtype)
    }

    /// Returns a reference to the underlying record of type `T` or `None` if it points
    /// to another record type.
    ///
    /// Note: for safety, this method calls [`has::<T>()`](Self::has). To avoid a
    /// duplicate check, use [`get_unchecked()`](Self::get_unchecked).
    pub fn get<T: HasRType>(&self) -> Option<&T> {
        if self.has::<T>() {
            // Safety: checked `rtype` in call to `has()`. Assumes the initial data based to
            // `RecordRef` is indeed a record.
            Some(unsafe { self.ptr.cast::<T>().as_ref() })
        } else {
            None
        }
    }

    /// Returns a reference to the underlying record of type `T` without checking if
    /// this object references a record of type `T`.
    ///
    /// For a safe alternative, see [`get()`](Self::get).
    ///
    /// # Safety
    /// The caller needs to validate this object points to a `T`.
    pub unsafe fn get_unchecked<T: HasRType>(&self) -> &T {
        debug_assert!(self.has::<T>());
        self.ptr.cast::<T>().as_ref()
    }
}
