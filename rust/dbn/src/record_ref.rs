//! The [`RecordRef`] struct for non-owning dynamically-typed references to DBN records.

use std::{fmt::Debug, hash, io::IoSlice, marker::PhantomData, mem, ptr::NonNull};

use crate::{
    record::{HasRType, Record, RecordHeader},
    rtype_dispatch, RecordEnum, RecordMut, RecordRefEnum,
};

/// A wrapper around a non-owning immutable reference to a DBN record. This wrapper
/// allows for mixing of record types and schemas and runtime record polymorphism.
///
/// Can hold any type implementing [`HasRType`] and acts similar to a `&dyn HasRType`
/// or `&dyn Record` but due to the design of DBN records, only a few methods require a
/// dynamic dispatch.
///
/// It has the [`has()`](Self::has) method for testing if the contained value is of a
/// particular type, and the inner value can be downcasted to specific record types via
/// the [`get()`](Self::get) method.
///
/// # Examples
/// ```
/// use dbn::{MboMsg, RecordRef, TradeMsg};
///
/// let mbo = MboMsg::default();
/// let rec = RecordRef::from(&mbo);
///
/// // This isn't a trade
/// assert!(!rec.has::<TradeMsg>());
/// // It's an MBO record
/// assert!(rec.has::<MboMsg>());
///
/// // `get()` can be used in `if let` chains:
/// if let Some(_trade) = rec.get::<TradeMsg>() {
///     panic!("Unexpected record type");
/// } else if let Some(mbo) = rec.get::<MboMsg>() {
///     println!("{mbo:?}");
/// }
/// ```
///
/// The common record header is directly accessible through the
/// [`header()`](Self::header) method.
#[derive(Copy, Clone)]
pub struct RecordRef<'a> {
    ptr: NonNull<RecordHeader>,
    /// Associates the object with the lifetime of the memory pointed to by `ptr`.
    _marker: PhantomData<&'a RecordHeader>,
}

/// A wrapper around a mutable reference to a DBN record. This wrapper
/// allows for mixing of record types and schemas, and runtime record polymorphism.
#[derive(Copy, Clone)]
pub struct RecordRefMut<'a> {
    ptr: NonNull<RecordHeader>,
    /// Associates the object with the lifetime of the memory pointed to by `ptr`.
    _marker: PhantomData<&'a RecordHeader>,
}

// Safety: RecordRef exhibits immutable reference semantics similar to &T.
// It should be safe to both send it across threads or access it simultaneously
// (since the data is immutable).
unsafe impl Send for RecordRef<'_> {}
unsafe impl Sync for RecordRef<'_> {}

// Safety: RecordRefMut exhibits mutable reference semantics similar to &mut T.
// It should be safe to send it across threads (unique ownership of the referent).
unsafe impl Send for RecordRefMut<'_> {}
unsafe impl Sync for RecordRefMut<'_> {}

impl<'a> RecordRef<'a> {
    /// Constructs a new reference to the DBN record in `buffer`.
    ///
    /// # Safety
    /// `buffer` should begin with a [`RecordHeader`] and contain a type implementing
    /// [`HasRType`].
    pub unsafe fn new(buffer: &'a [u8]) -> Self {
        debug_assert!(
            buffer.len() >= mem::size_of::<RecordHeader>(),
            "buffer of length {} is too short",
            buffer.len()
        );

        // Safety: casting to `*mut` to use `NonNull`, but `ptr` is still treated internally
        // as an immutable reference
        let raw_ptr = buffer.as_ptr() as *mut RecordHeader;

        // Check if alignment of pointer matches that of header (and all records)
        debug_assert_eq!(
            raw_ptr.align_offset(std::mem::align_of::<RecordHeader>()),
            0,
            "unaligned buffer passed to `RecordRef::new`"
        );
        let ptr = NonNull::new_unchecked(raw_ptr.cast::<RecordHeader>());
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Constructs a new reference to the DBN record.
    ///
    /// # Safety
    /// `header` must point to a valid DBN record.
    pub unsafe fn unchecked_from_header(header: *const RecordHeader) -> Self {
        Self {
            // `NonNull` requires `mut` but it is never mutated
            ptr: NonNull::new_unchecked(header.cast_mut()),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the object points to a record of type `T`.
    ///
    /// Usually paired with [`get()`](Self::get) or [`get_unchecked()`](Self::get_unchecked).
    ///
    /// <div class="warning">
    /// Only checks the <code>rtype</code> matches that of the type <code>T</code>. It does not check
    /// the length.
    /// </div>
    ///
    /// Use [`try_get()`](Self::try_get) when working with different versions of a DBN
    /// struct.
    ///
    /// # Examples
    /// ```
    /// use dbn::{OhlcvMsg, RecordRef, Schema, TradeMsg};
    ///
    /// let bar = OhlcvMsg::default_for_schema(Schema::Ohlcv1M);
    /// let rec = RecordRef::from(&bar);
    ///
    /// // This is a bar
    /// assert!(rec.has::<OhlcvMsg>());
    /// // It's not a trade
    /// assert!(!rec.has::<TradeMsg>());
    /// ```
    pub fn has<T: HasRType>(&self) -> bool {
        T::has_rtype(self.header().rtype)
    }

    /// Returns a reference to the underlying record of type `T` or `None` if it points
    /// to another record type.
    ///
    /// Note: for safety, this method calls [`has::<T>()`](Self::has). To avoid a
    /// duplicate check, use [`get_unchecked()`](Self::get_unchecked).
    ///
    /// # Panics
    /// This function will panic if the rtype indicates it's of type `T` but the encoded
    ///  length of the record is less than the size of `T`. Use [`try_get()`](Self::try_get)
    /// to more gracefully handle versioned structs and the optional presence of [`crate::WithTsOut`].
    ///
    /// # Examples
    /// ```
    /// use dbn::{BboMsg, RecordRef, Schema};
    ///
    /// let bbo = BboMsg::default_for_schema(Schema::Bbo1S);
    /// let rec = RecordRef::from(&bbo);
    ///
    /// if let Some(bbo) = rec.get::<BboMsg>() {
    ///     println!("{bbo:?}");
    /// }
    /// ```
    ///
    /// With versioned DBN structs
    /// ```should_panic
    /// use dbn::{v1, v2, RecordRef};
    ///
    /// // Initialize with version 1 definition
    /// let def = v1::InstrumentDefMsg::default();
    /// let rec = RecordRef::from(&def);
    /// // Try to extract a version 2 definition
    /// let _def = rec.get::<v2::InstrumentDefMsg>();
    /// ```
    pub fn get<T: HasRType>(&self) -> Option<&'a T> {
        if self.has::<T>() {
            assert!(
                self.record_size() >= mem::size_of::<T>(),
                "Malformed `{}` record: expected length of at least {} bytes, found {} bytes. \
                Confirm the DBN version in the Metadata header and the version upgrade policy",
                std::any::type_name::<T>(),
                mem::size_of::<T>(),
                self.record_size()
            );
            // Safety: checked `rtype` in call to `has()`. Assumes the initial data based to
            // `RecordRef` is indeed a record.
            Some(unsafe { self.ptr.cast::<T>().as_ref() })
        } else {
            None
        }
    }

    /// Like [`get()`](Self::get), but returns an error if the inner record is not a `T`
    /// or has the correct `rtype` for `T`, but insufficient `length`. Never panics.
    ///
    /// # Errors
    /// This function returns an error if does not hold a `T` or if its `rtype` matches
    /// `T`, but its `length` is too short.
    ///
    /// # Examples
    /// ```
    /// use dbn::{v1, v2, v3, RecordRef, WithTsOut};
    ///
    /// // Initialize with version 1 definition
    /// let def = v1::InstrumentDefMsg::default();
    /// let rec = RecordRef::from(&def);
    /// // Try to extract new versions of definitions
    /// assert!(rec.try_get::<v2::InstrumentDefMsg>().is_err());
    /// assert!(rec.try_get::<v3::InstrumentDefMsg>().is_err());
    ///
    /// rec.try_get::<v1::InstrumentDefMsg>().unwrap();
    ///
    /// // Also works with data that might have ts_out
    /// assert!(rec.try_get::<WithTsOut<v1::InstrumentDefMsg>>().is_err());
    /// ```
    pub fn try_get<T: HasRType>(&self) -> crate::Result<&'a T> {
        if self.has::<T>() {
            if self.record_size() >= mem::size_of::<T>() {
                // Safety: checked `rtype` in call to `has()` and size
                Ok(unsafe { self.ptr.cast::<T>().as_ref() })
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

    /// Returns a native Rust enum with a variant for each record type. This allows for
    /// pattern `match`ing.
    ///
    /// # Errors
    /// This function returns a conversion error if the rtype does not correspond with
    /// any known DBN record type.
    pub fn as_enum(&self) -> crate::Result<RecordRefEnum<'_>> {
        RecordRefEnum::try_from(*self)
    }

    /// Returns a reference to the underlying record of type `T` without checking if
    /// this object references a record of type `T`.
    ///
    /// For a safe alternative, see [`get()`](Self::get).
    ///
    /// # Safety
    /// The caller needs to validate this object points to a `T`.
    ///
    /// # Examples
    /// ```
    /// use dbn::{BboMsg, RecordRef, Schema};
    ///
    /// let bbo = BboMsg::default_for_schema(Schema::Bbo1S);
    /// let rec = RecordRef::from(&bbo);
    ///
    /// if rec.has::<BboMsg>() {
    ///     // SAFETY: checked rtype
    ///     println!("{:?}", unsafe { rec.get_unchecked::<BboMsg>() });
    /// }
    /// ```
    pub unsafe fn get_unchecked<T: HasRType>(&self) -> &'a T {
        debug_assert!(self.record_size() >= mem::size_of::<T>());
        self.ptr.cast::<T>().as_ref()
    }

    /// Creates an owned [`RecordBuf`](crate::RecordBuf) by copying the record bytes.
    ///
    /// # Examples
    /// ```
    /// use dbn::{MboMsg, RecordRef};
    ///
    /// let mbo = MboMsg::default();
    /// let rec_ref = RecordRef::from(&mbo);
    /// let owned = rec_ref.to_owned();
    /// assert!(owned == rec_ref);
    /// ```
    pub fn to_owned(&self) -> crate::RecordBuf {
        // All valid records fit within MAX_RECORD_LEN.
        crate::RecordBuf::try_from(*self).expect("record exceeds MAX_RECORD_LEN")
    }
}

impl<'a, R> From<&'a R> for RecordRef<'a>
where
    R: HasRType,
{
    /// Constructs a new reference to a DBN record.
    fn from(rec: &'a R) -> Self {
        Self {
            // Safety: `R` must be a record because it implements `HasRType`. Casting to `mut`
            // is required for `NonNull`, but it is never mutated.
            ptr: unsafe {
                NonNull::new_unchecked((rec.header() as *const RecordHeader).cast_mut())
            },
            _marker: PhantomData,
        }
    }
}

impl<'a> AsRef<[u8]> for RecordRef<'a> {
    fn as_ref(&self) -> &'a [u8] {
        // # Safety
        // Assumes the encoded record length is correct.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.record_size()) }
    }
}

impl<'a> Record for RecordRef<'a> {
    fn header(&self) -> &'a RecordHeader {
        // Safety: assumes `ptr` passes to a `RecordHeader`.
        unsafe { self.ptr.as_ref() }
    }

    fn raw_index_ts(&self) -> u64 {
        fn raw_index_ts<T: HasRType>(t: &T) -> u64 {
            t.raw_index_ts()
        }
        rtype_dispatch!(self, raw_index_ts()).unwrap_or_else(|_| self.header().ts_event)
    }
}

impl<'a> From<&'a RecordEnum> for RecordRef<'a> {
    fn from(rec_enum: &'a RecordEnum) -> Self {
        match rec_enum {
            RecordEnum::Mbo(rec) => Self::from(rec),
            RecordEnum::Trade(rec) => Self::from(rec),
            RecordEnum::Mbp1(rec) => Self::from(rec),
            RecordEnum::Mbp10(rec) => Self::from(rec),
            RecordEnum::Ohlcv(rec) => Self::from(rec),
            RecordEnum::Status(rec) => Self::from(rec),
            RecordEnum::InstrumentDef(rec) => Self::from(rec),
            RecordEnum::Imbalance(rec) => Self::from(rec),
            RecordEnum::Stat(rec) => Self::from(rec),
            RecordEnum::Error(rec) => Self::from(rec),
            RecordEnum::SymbolMapping(rec) => Self::from(rec),
            RecordEnum::System(rec) => Self::from(rec),
            RecordEnum::Cmbp1(rec) => Self::from(rec),
            RecordEnum::Bbo(rec) => Self::from(rec),
            RecordEnum::Cbbo(rec) => Self::from(rec),
        }
    }
}

impl<'a> From<RecordRefEnum<'a>> for RecordRef<'a> {
    fn from(rec_enum: RecordRefEnum<'a>) -> Self {
        match rec_enum {
            RecordRefEnum::Mbo(rec) => Self::from(rec),
            RecordRefEnum::Trade(rec) => Self::from(rec),
            RecordRefEnum::Mbp1(rec) => Self::from(rec),
            RecordRefEnum::Mbp10(rec) => Self::from(rec),
            RecordRefEnum::Ohlcv(rec) => Self::from(rec),
            RecordRefEnum::Status(rec) => Self::from(rec),
            RecordRefEnum::InstrumentDef(rec) => Self::from(rec),
            RecordRefEnum::Imbalance(rec) => Self::from(rec),
            RecordRefEnum::Stat(rec) => Self::from(rec),
            RecordRefEnum::Error(rec) => Self::from(rec),
            RecordRefEnum::SymbolMapping(rec) => Self::from(rec),
            RecordRefEnum::System(rec) => Self::from(rec),
            RecordRefEnum::Cmbp1(rec) => Self::from(rec),
            RecordRefEnum::Bbo(rec) => Self::from(rec),
            RecordRefEnum::Cbbo(rec) => Self::from(rec),
        }
    }
}

impl<'a> From<RecordRef<'a>> for IoSlice<'a> {
    fn from(rec: RecordRef<'a>) -> Self {
        // SAFETY: Assumes the encoded record length is correct.
        Self::new(unsafe {
            std::slice::from_raw_parts(rec.ptr.as_ptr() as *const u8, rec.record_size())
        })
    }
}

impl Debug for RecordRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordRef")
            .field(
                "ptr",
                &format_args!("{:?} --> {:?}", self.ptr, self.header()),
            )
            .finish()
    }
}

impl<'a> RecordRefMut<'a> {
    /// Constructs a new reference to the DBN record in `buffer`.
    ///
    /// # Safety
    /// `buffer` should begin with a [`RecordHeader`] and contain a type implementing
    /// [`HasRType`].
    pub unsafe fn new(buffer: &'a mut [u8]) -> Self {
        debug_assert!(buffer.len() >= mem::size_of::<RecordHeader>());

        // Safety: casting to `*mut` to use `NonNull`, but `ptr` is still treated internally
        // as an immutable reference
        let raw_ptr = buffer.as_ptr() as *mut RecordHeader;

        // Check if alignment of pointer matches that of header (and all records)
        debug_assert_eq!(
            raw_ptr.align_offset(std::mem::align_of::<RecordHeader>()),
            0
        );
        let ptr = NonNull::new_unchecked(raw_ptr.cast::<RecordHeader>());
        Self {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Constructs a new reference to the DBN record.
    ///
    /// # Safety
    /// `header` must point to a valid, mutable DBN record.
    pub unsafe fn unchecked_from_header(header: *mut RecordHeader) -> Self {
        Self {
            ptr: NonNull::new_unchecked(header),
            _marker: PhantomData,
        }
    }

    /// Returns `true` if the object points to a record of type `T`.
    pub fn has<T: HasRType>(&self) -> bool {
        T::has_rtype(self.header().rtype)
    }

    /// Returns a reference to the underlying record of type `T` or `None` if it points
    /// to another record type.
    ///
    /// # Panics
    /// This function will panic if the rtype indicates it's of type `T` but the encoded
    /// length of the record is less than the size of `T`. Use [`try_get()`](Self::try_get)
    /// to handle this gracefully.
    pub fn get<T: HasRType>(&self) -> Option<&'a T> {
        self.as_rec_ref().get()
    }

    /// Like [`get()`](Self::get), but returns an error instead of panicking when the
    /// rtype matches but the length is insufficient.
    ///
    /// # Errors
    /// This function returns an error if the buffer doesn't hold a `T`, or if the rtype
    /// matches but the length is too short.
    pub fn try_get<T: HasRType>(&self) -> crate::Result<&'a T> {
        self.as_rec_ref().try_get()
    }

    /// Returns a mutable reference to the underlying record of type `T` or `None` if it
    /// points to another record type.
    ///
    /// # Panics
    /// This function will panic if the rtype indicates it's of type `T` but the encoded
    /// length of the record is less than the size of `T`. Use
    /// [`try_get_mut()`](Self::try_get_mut) to handle this gracefully.
    pub fn get_mut<T: HasRType>(&self) -> Option<&'a mut T> {
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
            Some(unsafe { self.ptr.cast::<T>().as_mut() })
        } else {
            None
        }
    }

    /// Like [`get_mut()`](Self::get_mut), but returns an error instead of panicking when
    /// the rtype matches but the length is insufficient.
    ///
    /// # Errors
    /// This function returns an error if the buffer doesn't hold a `T`, or if the rtype
    /// matches but the length is too short.
    pub fn try_get_mut<T: HasRType>(&mut self) -> crate::Result<&'a mut T> {
        if self.has::<T>() {
            if self.record_size() >= mem::size_of::<T>() {
                // SAFETY: checked rtype and size.
                Ok(unsafe { self.ptr.cast::<T>().as_mut() })
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

    /// Returns an immutable reference to the underlying record of type `T` without
    /// checking if this object references a record of type `T`.
    ///
    /// For a safe alternative, see [`get()`](Self::get).
    ///
    /// # Safety
    /// The caller needs to validate this object points to a `T`.
    pub unsafe fn get_unchecked<T: HasRType>(&self) -> &'a T {
        debug_assert!(self.record_size() >= mem::size_of::<T>());
        self.ptr.cast::<T>().as_ref()
    }

    /// Returns a mutable reference to the underlying record of type `T` without
    /// checking if this object references a record of type `T`.
    ///
    /// For a safe alternative, see [`get_mut()`](Self::get_mut).
    ///
    /// # Safety
    /// The caller needs to validate this object points to a `T`.
    pub unsafe fn get_mut_unchecked<T: HasRType>(&mut self) -> &'a mut T {
        debug_assert!(self.record_size() >= mem::size_of::<T>());
        self.ptr.cast::<T>().as_mut()
    }

    /// Creates an owned [`RecordBuf`](crate::RecordBuf) by copying the record bytes.
    pub fn to_owned(&self) -> crate::RecordBuf {
        // All valid records fit within MAX_RECORD_LEN.
        crate::RecordBuf::try_from(self.as_rec_ref()).expect("record exceeds MAX_RECORD_LEN")
    }

    /// Returns an immutable [`RecordRef`] view of this mutable reference.
    pub fn as_rec_ref(&self) -> RecordRef<'a> {
        RecordRef {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<'a, R> From<&'a mut R> for RecordRefMut<'a>
where
    R: HasRType,
{
    /// Constructs a new reference to a DBN record.
    fn from(rec: &'a mut R) -> Self {
        Self {
            // Safety: `R` must be a record because it implements `HasRType`. Casting to `mut`
            // is required for `NonNull`, but it is never mutated.
            ptr: unsafe {
                NonNull::new_unchecked((rec.header() as *const RecordHeader).cast_mut())
            },
            _marker: PhantomData,
        }
    }
}

impl<'a> AsRef<[u8]> for RecordRefMut<'a> {
    fn as_ref(&self) -> &'a [u8] {
        // # Safety
        // Assumes the encoded record length is correct.
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as *const u8, self.record_size()) }
    }
}

impl<'a> Record for RecordRefMut<'a> {
    fn header(&self) -> &'a RecordHeader {
        // Safety: assumes `ptr` points to a `RecordHeader`.
        unsafe { self.ptr.as_ref() }
    }

    fn raw_index_ts(&self) -> u64 {
        fn raw_index_ts<T: HasRType>(t: &T) -> u64 {
            t.raw_index_ts()
        }
        rtype_dispatch!(self, raw_index_ts()).unwrap_or_else(|_| self.header().ts_event)
    }
}

impl<'a> RecordMut for RecordRefMut<'a> {
    fn header_mut(&mut self) -> &mut RecordHeader {
        // Safety: assumes `ptr` points to a `RecordHeader`.
        unsafe { self.ptr.as_mut() }
    }
}

impl Debug for RecordRefMut<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecordRefMut")
            .field(
                "ptr",
                &format_args!("{:?} --> {:?}", self.ptr, self.header()),
            )
            .finish()
    }
}

impl<'a, const CAP: usize> From<&'a crate::RecordBuf<CAP>> for RecordRef<'a> {
    fn from(buf: &'a crate::RecordBuf<CAP>) -> Self {
        buf.as_rec_ref()
    }
}

impl<'a, const CAP: usize> From<&'a mut crate::RecordBuf<CAP>> for RecordRefMut<'a> {
    fn from(buf: &'a mut crate::RecordBuf<CAP>) -> Self {
        buf.as_rec_ref_mut()
    }
}

impl<'a> From<RecordRefMut<'a>> for RecordRef<'a> {
    fn from(ref_mut: RecordRefMut<'a>) -> Self {
        ref_mut.as_rec_ref()
    }
}

impl hash::Hash for RecordRef<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl hash::Hash for RecordRefMut<'_> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}

impl PartialEq for RecordRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl Eq for RecordRef<'_> {}

impl<const CAP: usize> PartialEq<crate::RecordBuf<CAP>> for RecordRef<'_> {
    fn eq(&self, other: &crate::RecordBuf<CAP>) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl PartialEq<RecordRefMut<'_>> for RecordRef<'_> {
    fn eq(&self, other: &RecordRefMut<'_>) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl PartialEq for RecordRefMut<'_> {
    fn eq(&self, other: &Self) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl Eq for RecordRefMut<'_> {}

impl<const CAP: usize> PartialEq<crate::RecordBuf<CAP>> for RecordRefMut<'_> {
    fn eq(&self, other: &crate::RecordBuf<CAP>) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

impl PartialEq<RecordRef<'_>> for RecordRefMut<'_> {
    fn eq(&self, other: &RecordRef<'_>) -> bool {
        *self.as_ref() == *other.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::c_char;

    use crate::{
        enums::rtype, v1, v3, ErrorMsg, FlagSet, InstrumentDefMsg, MboMsg, Mbp10Msg, Mbp1Msg,
        OhlcvMsg, TradeMsg,
    };

    use super::*;

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
    fn test_header() {
        let target = RecordRef::from(&SOURCE_RECORD);
        assert_eq!(*target.header(), SOURCE_RECORD.hd);
    }

    #[test]
    fn test_fmt_debug() {
        let target = RecordRef::from(&SOURCE_RECORD);
        let string = format!("{target:?}");
        dbg!(&string);
        assert!(string.starts_with("RecordRef { ptr: 0x"));
        assert!(string.ends_with("--> RecordHeader { length: 14, rtype: Mbo, publisher_id: GlbxMdp3Glbx, instrument_id: 1, ts_event: 0 } }"));
    }

    #[test]
    fn test_has_and_get() {
        let target = RecordRef::from(&SOURCE_RECORD);
        assert!(!target.has::<Mbp1Msg>());
        assert!(!target.has::<Mbp10Msg>());
        assert!(!target.has::<TradeMsg>());
        assert!(!target.has::<ErrorMsg>());
        assert!(!target.has::<OhlcvMsg>());
        assert!(!target.has::<InstrumentDefMsg>());
        assert!(target.has::<MboMsg>());
        assert_eq!(*target.get::<MboMsg>().unwrap(), SOURCE_RECORD);
    }

    #[test]
    fn test_as_ref() {
        let target = RecordRef::from(&SOURCE_RECORD);
        let byte_slice = target.as_ref();
        assert_eq!(SOURCE_RECORD.record_size(), byte_slice.len());
        assert_eq!(target.record_size(), byte_slice.len());
    }

    #[should_panic]
    #[test]
    fn test_get_too_short() {
        let mut src = SOURCE_RECORD;
        src.hd.length -= 1;
        let target = RecordRef::from(&src);
        // panic due to unexpected length
        target.get::<MboMsg>();
    }

    #[should_panic]
    #[test]
    fn test_get_previous_ver() {
        let src = v1::InstrumentDefMsg::default();
        let target = RecordRef::from(&src);
        // panic due to `src` having shorter record length despite matching rtypes
        target.get::<v3::InstrumentDefMsg>();
    }

    #[test]
    fn test_try_get_previous_ver() {
        let src = v1::InstrumentDefMsg::default();
        let target = RecordRef::from(&src);
        assert!(
            matches!(target.try_get::<v3::InstrumentDefMsg>(), Err(e) if e.to_string().contains("has insufficient length"))
        );
    }

    #[test]
    fn niche() {
        assert_eq!(
            std::mem::size_of::<RecordRef>(),
            std::mem::size_of::<Option<RecordRef>>()
        );
        assert_eq!(
            std::mem::size_of::<RecordRef>(),
            std::mem::size_of::<usize>()
        );
    }

    #[test]
    fn test_record_ref_mut_get_delegates() {
        let mut mbo = SOURCE_RECORD;
        let target = RecordRefMut::from(&mut mbo);
        assert!(target.has::<MboMsg>());
        assert!(!target.has::<TradeMsg>());
        assert_eq!(*target.get::<MboMsg>().unwrap(), SOURCE_RECORD);
        assert!(target.get::<TradeMsg>().is_none());
    }

    #[test]
    fn test_record_ref_mut_try_get() {
        let mut def = v1::InstrumentDefMsg::default();
        let target = RecordRefMut::from(&mut def);
        target.try_get::<v1::InstrumentDefMsg>().unwrap();
        assert!(
            matches!(target.try_get::<v3::InstrumentDefMsg>(), Err(e) if e.to_string().contains("has insufficient length"))
        );
        assert!(
            matches!(target.try_get::<MboMsg>(), Err(e) if e.to_string().contains("has incorrect rtype"))
        );
    }

    #[test]
    fn test_record_ref_mut_try_get_mut() {
        let mut mbo = SOURCE_RECORD;
        let mut target = RecordRefMut::from(&mut mbo);
        let rec = target.try_get_mut::<MboMsg>().unwrap();
        rec.price = 42;
        assert_eq!(mbo.price, 42);
    }

    #[test]
    fn test_record_ref_mut_get_mut() {
        let mut mbo = SOURCE_RECORD;
        let target = RecordRefMut::from(&mut mbo);
        let rec = target.get_mut::<MboMsg>().unwrap();
        rec.size = 99;
        assert_eq!(mbo.size, 99);
    }

    #[test]
    fn test_record_ref_mut_to_owned() {
        let mut mbo = SOURCE_RECORD;
        let target = RecordRefMut::from(&mut mbo);
        let owned = target.to_owned();
        assert_eq!(*owned.get::<MboMsg>().unwrap(), SOURCE_RECORD);
    }

    #[test]
    fn test_record_ref_mut_as_rec_ref() {
        let mut mbo = SOURCE_RECORD;
        let target = RecordRefMut::from(&mut mbo);
        let rec_ref: RecordRef = target.as_rec_ref();
        assert_eq!(*rec_ref.get::<MboMsg>().unwrap(), SOURCE_RECORD);
    }

    #[test]
    fn test_from_record_ref_mut_to_record_ref() {
        let mut mbo = SOURCE_RECORD;
        let target = RecordRefMut::from(&mut mbo);
        let rec_ref: RecordRef = target.into();
        assert_eq!(*rec_ref.get::<MboMsg>().unwrap(), SOURCE_RECORD);
    }
}
