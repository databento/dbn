use super::ts_to_dt;
use crate::{Publisher, RType, RecordHeader};

/// Used for polymorphism around types all beginning with a [`RecordHeader`] where
/// `rtype` is the discriminant used for indicating the type of record.
pub trait Record: AsRef<[u8]> {
    /// Returns a reference to the `RecordHeader` that comes at the beginning of all
    /// record types.
    fn header(&self) -> &RecordHeader;

    /// Returns the size of the record in bytes.
    fn record_size(&self) -> usize {
        self.header().record_size()
    }

    /// Tries to convert the raw record type into an enum which is useful for exhaustive
    /// pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `rtype` field does not
    /// contain a valid, known [`RType`].
    fn rtype(&self) -> crate::Result<RType> {
        self.header().rtype()
    }

    /// Tries to convert the raw `publisher_id` into an enum which is useful for
    /// exhaustive pattern matching.
    ///
    /// # Errors
    /// This function returns an error if the `publisher_id` does not correspond with
    /// any known [`Publisher`].
    fn publisher(&self) -> crate::Result<Publisher> {
        self.header().publisher()
    }

    /// Returns the raw primary timestamp for the record.
    ///
    /// This timestamp should be used for sorting records as well as indexing into any
    /// symbology data structure.
    fn raw_index_ts(&self) -> u64 {
        self.header().ts_event
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
pub trait HasRType: Record + RecordMut {
    /// Returns `true` if `rtype` matches the value associated with the implementing type.
    fn has_rtype(rtype: u8) -> bool;
}
