//! Core traits for working with DBN records.
//!
//! - [`Record`]: read-only access to any record's header, size, timestamps, and raw bytes.
//! - [`RecordMut`]: mutable access to the record header.
//! - [`HasRType`]: implemented by concrete record types (e.g. [`MboMsg`](crate::MboMsg)),
//!   associates a static `rtype` used for downcasting via [`RecordRef`](crate::RecordRef).
//! - [`RecordHeaderKind`]: the header layouts a record can begin with, abstracted so
//!   framing and downcasting work on any DBN version.

use super::ts_to_dt;
use crate::{Publisher, RType, RecordHeader};

/// Used for polymorphism around types all beginning with a record header where `rtype`
/// is the discriminant used for indicating the type of record. [`Header`](Self::Header)
/// names which header layout, and so which DBN version.
///
/// All record types are plain old data held in sequential memory, and therefore
/// implement `AsRef<[u8]>` for simple serialization to bytes.
///
/// [`RecordRef`](crate::RecordRef) acts similar to a `&dyn Record`.
pub trait Record: AsRef<[u8]> {
    /// The layout of the record header this type begins with. Ties a record to a DBN
    /// version so [`RecordRef`](crate::RecordRef) downcasts can't mix a v1-v3 type with
    /// a v4 header (or vice versa).
    type Header: RecordHeaderKind;

    /// Returns the size of the record in bytes.
    fn record_size(&self) -> usize;

    /// Tries to convert the raw record type into an enum which is useful for exhaustive
    /// pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `rtype` field does not
    /// contain a valid, known [`RType`].
    fn rtype(&self) -> crate::Result<RType>;

    /// Returns the raw record type.
    fn raw_rtype(&self) -> u16;

    /// Returns the record's publisher ID.
    fn publisher_id(&self) -> u16;

    /// Tries to convert the raw `publisher_id` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `publisher_id` does not correspond with
    /// any known [`Publisher`].
    fn publisher(&self) -> crate::Result<Publisher>;

    /// Returns the record's instrument ID.
    fn instrument_id(&self) -> u64;

    /// Returns the raw event timestamp from the record header.
    fn raw_ts_event(&self) -> u64;

    /// Returns the raw primary timestamp for the record.
    ///
    /// This timestamp should be used for sorting records as well as indexing into any
    /// symbology data structure.
    fn raw_index_ts(&self) -> u64 {
        self.raw_ts_event()
    }

    /// Returns the primary timestamp for the record. Returns `None` if the primary
    /// timestamp contains the sentinel value for a null timestamp.
    ///
    /// This timestamp should be used for sorting records as well as indexing into any
    /// symbology data structure.
    fn index_ts(&self) -> Option<time::OffsetDateTime> {
        ts_to_dt(self.raw_index_ts())
    }

    /// Returns the primary date for the record; the date component of the primary
    /// timestamp (`index_ts()`). Returns `None` if the primary timestamp contains the
    /// sentinel value for a null timestamp.
    fn index_date(&self) -> Option<time::Date> {
        self.index_ts().map(|dt| dt.date())
    }
}

/// Used for polymorphism around mutable types beginning with a [`RecordHeader`].
pub trait RecordMut {
    /// Returns a mutable reference to the `RecordHeader` that comes at the beginning of
    /// all record types.
    fn header_mut(&mut self) -> &mut RecordHeader;
}

/// An extension of the [`Record`] trait for types with a static [`RType`]. Used for
/// determining if a rtype matches a type.
///
/// Because of the static function requirement, this trait is implemented by all concrete record
/// types like [`MboMsg`](crate::MboMsg), but not by [`RecordRef`](crate::RecordRef), which can reference a record of
/// dynamic type.
///
/// While not _dyn compatible_, [`RecordRef`](crate::RecordRef) acts like a `&dyn HasRType`.
pub trait HasRType: Record {
    /// Returns `true` if `rtype` matches the value associated with the implementing type.
    fn has_rtype(rtype: u16) -> bool;
}

/// Abstracts over the DBN record header layouts so encoders, decoders, and
/// [`RecordRef`](crate::RecordRef) can frame a record and downcast it without knowing
/// which DBN version it came from.
pub trait RecordHeaderKind: private::Sealed + Copy + std::fmt::Debug {
    /// Returns the size of the whole record in bytes, derived from the header's `length`.
    fn record_size(&self) -> usize;

    /// Returns the raw record type.
    fn raw_rtype(&self) -> u16;
}

impl RecordHeaderKind for RecordHeader {
    fn record_size(&self) -> usize {
        RecordHeader::record_size(self)
    }

    fn raw_rtype(&self) -> u16 {
        self.rtype as u16
    }
}

impl RecordHeaderKind for crate::v4::RecordHeader {
    fn record_size(&self) -> usize {
        crate::v4::RecordHeader::record_size(self)
    }

    fn raw_rtype(&self) -> u16 {
        self.rtype
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::RecordHeader {}
    impl Sealed for crate::v4::RecordHeader {}
}
