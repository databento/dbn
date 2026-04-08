//! The [`RecordBuf`] struct for owning a DBN record of a dynamic type.
//!
//! `RecordBuf` is a stack-allocated buffer that can hold any DBN record type. It
//! provides owned, dynamically-typed storage for records, complementing
//! [`RecordRef`] (borrowed, dynamic) and concrete types like [`MboMsg`](crate::MboMsg)
//! (owned, static).
//!
//! A `RecordBuf` always holds a valid record. Use `Option<RecordBuf>` where absence
//! is needed.
//!
//! The const-generic parameter `CAP` controls the maximum record size the buffer can
//! hold. It defaults to [`MAX_RECORD_LEN`], which fits any current DBN record type.
//!
//! # When to use which type
//!
//! - [`RecordRef`]: borrowing a record of unknown type (zero-copy)
//! - [`RecordBuf`]: owning a record of unknown type (stack-allocated)
//! - [`RecordEnum`](crate::RecordEnum) / [`RecordRefEnum`]: exhaustive pattern matching
//!   over all known record types
//! - Concrete types (`MboMsg`, `TradeMsg`, etc.): when the type is known at compile time

use std::{fmt::Debug, hash, io::IoSlice, mem};

use crate::{
    rtype_dispatch, HasRType, Record, RecordHeader, RecordMut, RecordRef, RecordRefEnum,
    RecordRefMut, MAX_RECORD_LEN,
};

/// An owned buffer that holds a DBN record of a dynamic type.
///
/// The const-generic parameter `CAP` controls the byte capacity of the buffer,
/// defaulting to [`MAX_RECORD_LEN`]. A `RecordBuf` always contains a valid record;
/// use `Option<RecordBuf>` to represent the absence of a record.
///
/// # Examples
/// ```
/// use dbn::{MboMsg, RecordBuf, RecordRef, TradeMsg};
///
/// let mbo = MboMsg::default();
/// let buf: RecordBuf = RecordBuf::from(mbo);
///
/// assert!(buf.has::<MboMsg>());
/// assert!(!buf.has::<TradeMsg>());
///
/// if let Some(mbo) = buf.get::<MboMsg>() {
///     println!("{mbo:?}");
/// }
/// ```
// TODO: once `generic_const_exprs` stabilizes, replace the union with
// `NonZeroU8 + [u8; CAP - 1]` for niche optimization on `Option<RecordBuf>`.
#[derive(Clone)]
#[cfg_attr(feature = "trivial_copy", derive(Copy))]
#[repr(align(8))]
pub struct RecordBuf<const CAP: usize = MAX_RECORD_LEN>(Repr<CAP>);

#[derive(Clone, Copy)]
union Repr<const CAP: usize> {
    hd: RecordHeader,
    buf: [u8; CAP],
}

impl<const CAP: usize> RecordBuf<CAP> {
    /// Returns the compile-time capacity of the buffer, i.e. the size of the largest
    /// record it can hold.
    pub const fn capacity() -> usize {
        CAP
    }

    /// Returns an immutable reference to the record as a [`RecordRef`].
    pub fn as_rec_ref(&self) -> RecordRef<'_> {
        // SAFETY: `RecordBuf` always holds a valid record with a valid header.
        unsafe { RecordRef::new(self.as_ref()) }
    }

    /// Returns a mutable reference to the record as a [`RecordRefMut`].
    pub fn as_rec_ref_mut(&mut self) -> RecordRefMut<'_> {
        // SAFETY: `RecordBuf` always holds a valid record with a valid header.
        unsafe { RecordRefMut::new(self.raw_buf_mut()) }
    }

    /// Returns a [`RecordRefEnum`] for exhaustive pattern matching.
    ///
    /// # Errors
    /// Returns an error if the rtype does not correspond to any known DBN record type.
    pub fn as_enum(&self) -> crate::Result<RecordRefEnum<'_>> {
        RecordRefEnum::try_from(self.as_rec_ref())
    }

    /// Upgrades the record from type `F` to type `T` in place.
    ///
    /// # Errors
    /// This function returns an error if the buffer doesn't contain a record of type `F`.
    ///
    /// # Examples
    /// ```
    /// use dbn::{v1, v3, RecordBuf};
    ///
    /// let def = v1::InstrumentDefMsg::default();
    /// let mut buf: RecordBuf = RecordBuf::from(def);
    /// buf.upgrade::<v1::InstrumentDefMsg, v3::InstrumentDefMsg>().unwrap();
    /// assert!(buf.has::<v3::InstrumentDefMsg>());
    /// ```
    pub fn upgrade<F, T>(&mut self) -> crate::Result<()>
    where
        F: HasRType,
        T: HasRType,
        T: for<'a> From<&'a F>,
    {
        let upgraded = T::from(self.try_get::<F>()?);
        self.set(upgraded);
        Ok(())
    }

    /// Copies the given record into the buffer, replacing any previous contents.
    ///
    /// # Examples
    /// ```
    /// use dbn::{MboMsg, RecordBuf, TradeMsg};
    ///
    /// let mbo = MboMsg::default();
    /// let mut buf: RecordBuf = RecordBuf::from(mbo);
    /// assert!(buf.has::<MboMsg>());
    ///
    /// let trade = TradeMsg::default();
    /// buf.set(trade);
    /// assert!(buf.has::<TradeMsg>());
    /// ```
    pub fn set<T>(&mut self, other: T)
    where
        T: HasRType,
    {
        const {
            assert!(
                mem::size_of::<T>() <= CAP,
                "record size exceeds buffer capacity",
            );
        }
        let size = other.record_size();
        debug_assert!(
            size <= CAP,
            "record_size ({size}) exceeds buffer capacity ({CAP})"
        );
        // SAFETY: the compile-time assert guarantees `size_of::<T>() <= CAP`. A
        // well-formed record satisfies `record_size() <= size_of::<T>()`, giving
        // `size <= CAP`. Accessing the union `buf` field requires unsafe.
        unsafe {
            self.0.buf[..size].copy_from_slice(other.as_ref());
            self.0.buf[size..].fill(0);
        }
    }

    /// Returns `true` if the buffer holds a record of type `T`.
    pub fn has<T: HasRType>(&self) -> bool {
        T::has_rtype(self.header().rtype)
    }

    /// Returns a reference to the inner record of type `T`, or `None` if the buffer
    /// holds a different record type.
    ///
    /// # Panics
    /// This function panics if the rtype matches `T` but the encoded length is less
    /// than the size of `T`. Use [`try_get()`](Self::try_get) to handle this gracefully.
    ///
    /// # Examples
    /// ```
    /// use dbn::{MboMsg, RecordBuf};
    ///
    /// let mbo = MboMsg::default();
    /// let buf: RecordBuf = RecordBuf::from(mbo);
    ///
    /// if let Some(rec) = buf.get::<MboMsg>() {
    ///     println!("{rec:?}");
    /// }
    /// ```
    pub fn get<T: HasRType>(&self) -> Option<&T> {
        if self.has::<T>() {
            assert!(
                self.record_size() >= mem::size_of::<T>(),
                "Malformed `{}` record: expected length of at least {} bytes, found {} bytes. \
                Confirm the DBN version in the Metadata header and the version upgrade policy",
                std::any::type_name::<T>(),
                mem::size_of::<T>(),
                self.record_size()
            );
            // SAFETY: checked rtype and size. `Repr` is a union starting at the same
            // address, and `RecordBuf` is aligned to 8 bytes.
            Some(unsafe { std::mem::transmute::<&Repr<CAP>, &T>(&self.0) })
        } else {
            None
        }
    }

    /// Returns a mutable reference to the inner record of type `T`, or `None` if the
    /// buffer holds a different record type.
    ///
    /// # Panics
    /// This function panics if the rtype matches `T` but the encoded length is less
    /// than the size of `T`. Use [`try_get_mut()`](Self::try_get_mut) to handle this
    /// gracefully.
    pub fn get_mut<T: HasRType>(&mut self) -> Option<&mut T> {
        if self.has::<T>() {
            assert!(
                self.record_size() >= mem::size_of::<T>(),
                "Malformed `{}` record: expected length of at least {} bytes, found {} bytes. \
                Confirm the DBN version in the Metadata header and the version upgrade policy",
                std::any::type_name::<T>(),
                mem::size_of::<T>(),
                self.record_size()
            );
            // SAFETY: checked rtype and size.
            Some(unsafe { std::mem::transmute::<&mut Repr<CAP>, &mut T>(&mut self.0) })
        } else {
            None
        }
    }

    /// Like [`get()`](Self::get), but returns an error instead of panicking when the
    /// rtype matches but the length is insufficient.
    ///
    /// # Errors
    /// This function returns an error if the buffer doesn't hold a `T`, or if the rtype
    /// matches but the length is too short (e.g. an older version of the record).
    ///
    /// # Examples
    /// ```
    /// use dbn::{v1, v3, RecordBuf};
    ///
    /// let def = v1::InstrumentDefMsg::default();
    /// let buf: RecordBuf = RecordBuf::from(def);
    ///
    /// // v1 is too short for v3
    /// assert!(buf.try_get::<v3::InstrumentDefMsg>().is_err());
    /// // v1 works
    /// buf.try_get::<v1::InstrumentDefMsg>().unwrap();
    /// ```
    pub fn try_get<T: HasRType>(&self) -> crate::Result<&T> {
        if self.has::<T>() {
            if self.record_size() >= mem::size_of::<T>() {
                // SAFETY: checked rtype and size.
                Ok(unsafe { std::mem::transmute::<&Repr<CAP>, &T>(&self.0) })
            } else {
                Err(crate::Error::conversion::<T>(format!(
                    "{self:?} has insufficient length, may be an earlier version of this record"
                )))
            }
        } else {
            Err(crate::Error::conversion::<T>(format!(
                "{self:?} has incorrect rtype"
            )))
        }
    }

    /// Like [`get_mut()`](Self::get_mut), but returns an error instead of panicking when
    /// the rtype matches but the length is insufficient.
    ///
    /// # Errors
    /// This function returns an error if the buffer doesn't hold a `T`, or if the rtype
    /// matches but the length is too short.
    pub fn try_get_mut<T: HasRType>(&mut self) -> crate::Result<&mut T> {
        if self.has::<T>() {
            if self.record_size() >= mem::size_of::<T>() {
                // SAFETY: checked rtype and size.
                Ok(unsafe { std::mem::transmute::<&mut Repr<CAP>, &mut T>(&mut self.0) })
            } else {
                Err(crate::Error::conversion::<T>(format!(
                    "{self:?} has insufficient length, may be an earlier version of this record"
                )))
            }
        } else {
            Err(crate::Error::conversion::<T>(format!(
                "{self:?} has incorrect rtype"
            )))
        }
    }

    /// Returns a reference to the inner record of type `T` without checking the rtype.
    ///
    /// For a safe alternative, see [`get()`](Self::get).
    ///
    /// # Safety
    /// The caller must ensure the buffer holds a record of type `T`.
    pub unsafe fn get_unchecked<T: HasRType>(&self) -> &T {
        debug_assert!(self.record_size() >= mem::size_of::<T>());
        // SAFETY: caller guarantees the buffer holds a `T`; `debug_assert` checks size.
        // Union field access and raw pointer dereference.
        self.0.buf.as_ptr().cast::<T>().as_ref().unwrap_unchecked()
    }

    /// Returns a mutable reference to the inner record of type `T` without checking the
    /// rtype.
    ///
    /// For a safe alternative, see [`get_mut()`](Self::get_mut).
    ///
    /// # Safety
    /// The caller must ensure the buffer holds a record of type `T`.
    pub unsafe fn get_unchecked_mut<T: HasRType>(&mut self) -> &mut T {
        debug_assert!(self.record_size() >= mem::size_of::<T>());
        // SAFETY: caller guarantees the buffer holds a `T`; `debug_assert` checks size.
        // Union field access and raw pointer dereference.
        self.0
            .buf
            .as_mut_ptr()
            .cast::<T>()
            .as_mut()
            .unwrap_unchecked()
    }
}

impl<const CAP: usize> Record for RecordBuf<CAP> {
    fn header(&self) -> &RecordHeader {
        // SAFETY: `RecordBuf` always holds a valid record. The `hd` field of the union
        // is always valid because every record starts with a `RecordHeader`.
        unsafe { &self.0.hd }
    }

    fn raw_index_ts(&self) -> u64 {
        fn raw_index_ts<T: HasRType>(t: &T) -> u64 {
            t.raw_index_ts()
        }
        rtype_dispatch!(self, raw_index_ts()).unwrap_or_else(|_| self.header().ts_event)
    }
}

impl<const CAP: usize> RecordMut for RecordBuf<CAP> {
    fn header_mut(&mut self) -> &mut RecordHeader {
        // SAFETY: same as `header()`.
        unsafe { &mut self.0.hd }
    }
}

impl<const CAP: usize> AsRef<[u8]> for RecordBuf<CAP> {
    fn as_ref(&self) -> &[u8] {
        // SAFETY: `buf` is always fully initialized (every constructor writes all bytes).
        // `record_size()` is derived from the header `length` field set on construction,
        // and is always <= CAP.
        unsafe { std::slice::from_raw_parts(self.0.buf.as_ptr(), self.record_size()) }
    }
}

impl<const CAP: usize> RecordBuf<CAP> {
    /// Returns a mutable slice of the full buffer (`CAP` bytes), suitable for use as a
    /// raw write target (e.g. reading record bytes directly from a decoder). After writing,
    /// the caller must ensure the header's `length` field correctly reflects the record size.
    pub fn raw_buf_mut(&mut self) -> &mut [u8; CAP] {
        // SAFETY: the union's `buf` field covers the full `CAP` bytes.
        unsafe { &mut self.0.buf }
    }
}

impl<T, const CAP: usize> From<T> for RecordBuf<CAP>
where
    T: HasRType,
{
    /// Creates a `RecordBuf` by copying the record into the buffer. The record type `T`
    /// must fit within `CAP`; this is enforced at compile time.
    fn from(value: T) -> Self {
        const {
            assert!(
                mem::size_of::<T>() <= CAP,
                "record size exceeds buffer capacity"
            )
        };
        let mut buf = [0u8; CAP];
        buf[..value.record_size()].copy_from_slice(value.as_ref());
        Self(Repr { buf })
    }
}

impl<'a, const CAP: usize> From<&'a RecordBuf<CAP>> for IoSlice<'a> {
    fn from(rec: &'a RecordBuf<CAP>) -> Self {
        Self::new(rec.as_ref())
    }
}

impl<const A: usize, const B: usize> PartialEq<RecordBuf<B>> for RecordBuf<A> {
    fn eq(&self, other: &RecordBuf<B>) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<const CAP: usize> Eq for RecordBuf<CAP> {}

impl<const CAP: usize> hash::Hash for RecordBuf<CAP> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl<const CAP: usize> PartialEq<RecordRef<'_>> for RecordBuf<CAP> {
    fn eq(&self, other: &RecordRef<'_>) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl<const CAP: usize> PartialEq<RecordRefMut<'_>> for RecordBuf<CAP> {
    fn eq(&self, other: &RecordRefMut<'_>) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl<const CAP: usize> Debug for RecordBuf<CAP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_rec<T: HasRType + Debug>(t: &T, debug: &mut std::fmt::DebugStruct) {
            debug.field("buf", &t);
        }
        let mut debug = f.debug_struct("RecordBuf");
        match rtype_dispatch!(self, fmt_rec(&mut debug)) {
            Ok(_) => debug.finish(),
            Err(_) => debug.field("hd", self.header()).finish_non_exhaustive(),
        }
    }
}

impl<const CAP: usize> TryFrom<RecordRef<'_>> for RecordBuf<CAP> {
    type Error = crate::Error;

    /// Creates a `RecordBuf` by copying bytes from a [`RecordRef`].
    ///
    /// # Errors
    /// Returns an error if the record is too large for the buffer's capacity.
    fn try_from(rec_ref: RecordRef<'_>) -> Result<Self, Self::Error> {
        if rec_ref.record_size() > CAP {
            Err(crate::Error::conversion::<Self>(format!(
                "{rec_ref:?} is too long for the RecordBuf's capacity"
            )))
        } else {
            let mut buf = [0; CAP];
            buf[..rec_ref.record_size()].copy_from_slice(rec_ref.as_ref());
            Ok(Self(Repr { buf }))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_char, io::IoSlice};

    use crate::{
        enums::rtype, v1, v3, FlagSet, MboMsg, RecordHeader, RecordRef, RecordRefEnum,
        RecordRefMut, TradeMsg, MAX_RECORD_LEN,
    };

    use super::*;

    /// Default-capacity `RecordBuf` for test annotations.
    type Buf = RecordBuf;

    const SOURCE_RECORD: MboMsg = MboMsg {
        hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 1, 0),
        order_id: 17,
        price: 0,
        size: 32,
        flags: FlagSet::empty(),
        channel_id: 1,
        action: 'A' as c_char,
        side: 'B' as c_char,
        ts_recv: 0,
        ts_in_delta: 160,
        sequence: 1067,
    };

    #[test]
    fn round_trip() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        let rec = buf.get::<MboMsg>().expect("should contain MboMsg");
        assert_eq!(*rec, SOURCE_RECORD);
    }

    #[test]
    fn wrong_type_returns_none() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(buf.has::<MboMsg>());
        assert!(!buf.has::<TradeMsg>());
        assert!(buf.get::<TradeMsg>().is_none());
    }

    #[test]
    fn try_get_insufficient_length() {
        let def = v1::InstrumentDefMsg::default();
        let buf: Buf = RecordBuf::from(def);
        let err = buf.try_get::<v3::InstrumentDefMsg>().unwrap_err();
        assert!(
            err.to_string().contains("has insufficient length"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn try_from_record_ref_capacity_overflow() {
        let mbo = SOURCE_RECORD;
        let rec_ref = RecordRef::from(&mbo);
        let result = RecordBuf::<4>::try_from(rec_ref);
        assert!(result.is_err());
    }

    #[test]
    fn upgrade_v1_to_v3() {
        let def = v1::InstrumentDefMsg::default();
        let mut buf: Buf = RecordBuf::from(def);
        assert!(buf.has::<v1::InstrumentDefMsg>());
        buf.upgrade::<v1::InstrumentDefMsg, v3::InstrumentDefMsg>()
            .unwrap();
        assert!(buf.has::<v3::InstrumentDefMsg>());
    }

    #[test]
    fn partial_eq_same_capacity() {
        let buf1: Buf = RecordBuf::from(SOURCE_RECORD);
        let buf2: Buf = RecordBuf::from(SOURCE_RECORD);
        assert_eq!(buf1, buf2);

        let other: Buf = RecordBuf::from(TradeMsg::default());
        assert_ne!(buf1, other);
    }

    #[test]
    fn partial_eq_cross_capacity() {
        let buf_default: Buf = RecordBuf::from(SOURCE_RECORD);
        let buf_small = RecordBuf::<256>::from(SOURCE_RECORD);
        assert!(buf_default == buf_small);
    }

    #[test]
    fn partial_eq_with_record_ref() {
        let mbo = SOURCE_RECORD;
        let buf: Buf = RecordBuf::from(mbo);
        let mbo2 = SOURCE_RECORD;
        let rec_ref = RecordRef::from(&mbo2);
        assert!(buf == rec_ref);
    }

    #[test]
    fn set_replaces_record() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(buf.has::<MboMsg>());

        let trade = TradeMsg::default();
        buf.set(trade);
        assert!(buf.has::<TradeMsg>());
        assert!(!buf.has::<MboMsg>());
    }

    #[test]
    fn get_mut_returns_mutable_ref() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        let rec = buf.get_mut::<MboMsg>().expect("should contain MboMsg");
        rec.order_id = 42;
        assert_eq!(buf.get::<MboMsg>().unwrap().order_id, 42);
    }

    #[test]
    fn get_mut_wrong_type_returns_none() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(buf.get_mut::<TradeMsg>().is_none());
    }

    #[test]
    fn try_get_mut_wrong_rtype() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        let err = buf.try_get_mut::<TradeMsg>().unwrap_err();
        assert!(
            err.to_string().contains("has incorrect rtype"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn try_get_mut_insufficient_length() {
        let def = v1::InstrumentDefMsg::default();
        let mut buf: Buf = RecordBuf::from(def);
        let err = buf.try_get_mut::<v3::InstrumentDefMsg>().unwrap_err();
        assert!(
            err.to_string().contains("has insufficient length"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn get_unchecked_returns_correct_record() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(buf.has::<MboMsg>());
        // SAFETY: checked rtype with `has`.
        let rec = unsafe { buf.get_unchecked::<MboMsg>() };
        assert_eq!(*rec, SOURCE_RECORD);
    }

    #[test]
    fn get_unchecked_mut_returns_correct_record() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(buf.has::<MboMsg>());
        // SAFETY: checked rtype with `has`.
        unsafe { buf.get_unchecked_mut::<MboMsg>() }.order_id = 99;
        assert_eq!(buf.get::<MboMsg>().unwrap().order_id, 99);
    }

    #[test]
    fn as_rec_ref_mut_allows_mutation() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        buf.as_rec_ref_mut().get_mut::<MboMsg>().unwrap().order_id = 77;
        assert_eq!(buf.get::<MboMsg>().unwrap().order_id, 77);
    }

    #[test]
    fn io_slice_spans_record_bytes_only() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        let slice = IoSlice::from(&buf);
        assert_eq!(slice.len(), buf.record_size());
        assert!(slice.len() < MAX_RECORD_LEN);
    }

    #[test]
    fn as_ref_returns_record_bytes_only() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert_eq!(buf.as_ref().len(), buf.record_size());
        assert!(buf.as_ref().len() < MAX_RECORD_LEN);
    }

    #[test]
    fn try_get_incorrect_rtype_error() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        let err = buf.try_get::<TradeMsg>().unwrap_err();
        assert!(
            err.to_string().contains("has incorrect rtype"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn partial_eq_with_record_ref_mut() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        let mut mbo = SOURCE_RECORD;
        let ref_mut = RecordRefMut::from(&mut mbo);
        assert!(buf == ref_mut);
    }

    #[test]
    fn upgrade_wrong_type_returns_error() {
        let mut buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(buf
            .upgrade::<v1::InstrumentDefMsg, v3::InstrumentDefMsg>()
            .is_err());
    }

    #[test]
    fn as_enum_dispatches_correctly() {
        let buf: Buf = RecordBuf::from(SOURCE_RECORD);
        assert!(matches!(buf.as_enum().unwrap(), RecordRefEnum::Mbo(_)));
    }

    #[test]
    fn set_clears_trailing_bytes() {
        // Start with the largest record type to fill the buffer, then replace with a
        // smaller one and verify the tail is zeroed.
        let def = v3::InstrumentDefMsg::default();
        let mut buf: Buf = RecordBuf::from(def);
        buf.set(SOURCE_RECORD);
        let record_size = buf.record_size();
        assert!(buf.raw_buf_mut()[record_size..].iter().all(|&b| b == 0));
    }
}
