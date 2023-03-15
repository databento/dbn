use std::{marker::PhantomData, mem, ptr::NonNull};

use crate::record::{HasRType, RecordHeader};

/// A wrapper around a non-owning immutable reference to a DBN record. This wrapper
/// allows for mixing of record types and schemas, and runtime record polymorphism.
#[derive(Clone, Debug)]
pub struct RecordRef<'a> {
    ptr: NonNull<RecordHeader>,
    /// Associates the object with the lifetime of the memory pointed to by `ptr`.
    _marker: PhantomData<&'a RecordHeader>,
}

impl<'a> RecordRef<'a> {
    /// Constructs a new reference to the DBN record in `buffer`.
    ///
    /// # Safety
    /// `buffer` should begin with a [`RecordHeader`] and contain a type implementing
    /// [`HasRType`].
    pub unsafe fn new(buffer: &'a [u8]) -> Self {
        debug_assert!(buffer.len() >= mem::size_of::<RecordHeader>());

        // Safety: casting to `*mut` to use `NonNull`, but `ptr` is still treated internally
        // as an immutable reference
        let raw_ptr = buffer.as_ptr() as *mut RecordHeader;

        let ptr = NonNull::new_unchecked(raw_ptr.cast::<RecordHeader>());
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

#[cfg(test)]
mod tests {
    use std::ffi::c_char;

    use crate::{
        enums::rtype,
        record::{ErrorMsg, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg, OhlcvMsg, TradeMsg},
    };

    use super::*;

    unsafe fn as_mut_u8_slice<T: Sized + 'static>(data: &mut T) -> &mut [u8] {
        std::slice::from_raw_parts_mut(data as *mut T as *mut u8, mem::size_of::<T>())
    }

    const SOURCE_RECORD: MboMsg = MboMsg {
        hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 1, 0),
        order_id: 17,
        price: 0,
        size: 32,
        flags: 0,
        channel_id: 1,
        action: 'A' as c_char,
        side: 'B' as c_char,
        ts_recv: 0,
        ts_in_delta: 160,
        sequence: 1067,
    };

    #[test]
    fn test_header() {
        let mut source = SOURCE_RECORD;
        let target = unsafe { RecordRef::new(as_mut_u8_slice(&mut source)) };
        assert_eq!(*target.header(), SOURCE_RECORD.hd);
    }

    #[test]
    fn test_has_and_get() {
        let mut source = SOURCE_RECORD;
        let target = unsafe { RecordRef::new(as_mut_u8_slice(&mut source)) };
        assert!(!target.has::<Mbp1Msg>());
        assert!(!target.has::<Mbp10Msg>());
        assert!(!target.has::<TradeMsg>());
        assert!(!target.has::<ErrorMsg>());
        assert!(!target.has::<OhlcvMsg>());
        assert!(!target.has::<InstrumentDefMsg>());
        assert!(target.has::<MboMsg>());
        assert_eq!(*unsafe { target.get_unchecked::<MboMsg>() }, SOURCE_RECORD);
    }
}
