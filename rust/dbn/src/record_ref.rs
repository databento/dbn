//! The [`RecordRef`] struct for non-owning dynamically-typed references to DBN records.

use std::{fmt::Debug, io::IoSlice, marker::PhantomData, mem, ptr::NonNull};

use crate::{
    record::{HasRType, Record, RecordHeader},
    rtype_dispatch, RecordEnum, RecordRefEnum,
};

/// A wrapper around a non-owning immutable reference to a DBN record. This wrapper
/// allows for mixing of record types and schemas and runtime record polymorphism.
///
/// It has the [`has()`](Self::has) method for testing if the contained value is of a
/// particular type, and the inner value can be downcasted to specific record types via
/// the [`get()`](Self::get) method.
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

// Safety: RecordRef exhibits immutable reference semantics similar to &T.
// It should be safe to both send it across threads or access it simultaneously
// (since the data is immutable).
unsafe impl Send for RecordRef<'_> {}
unsafe impl Sync for RecordRef<'_> {}

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
    ///  length of the record is less than the size of `T`.
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
    /// or has the correct `rtype` for `T`, but insufficient `length`. Never panics
    ///
    /// # Errors
    /// This function returns an error if does not hold a `T` or if its `rtype` matches
    /// `T`, but its `length` is too short.
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
    pub unsafe fn get_unchecked<T: HasRType>(&self) -> &'a T {
        debug_assert!(self.record_size() >= mem::size_of::<T>());
        self.ptr.cast::<T>().as_ref()
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
        // Safety: Assumes the encoded record length is correct.
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
}
