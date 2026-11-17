use crate::{HasRType, Record, RecordHeader};

/// A [`Record`] that begins with the [`RecordHeader`] shared by DBN versions 1 through 3.
pub trait RecordV1: Record<Header = RecordHeader> {}

impl<T: Record<Header = RecordHeader>> RecordV1 for T {}

/// A [`HasRType`] record type that begins with the [`RecordHeader`] shared by DBN
/// versions 1 through 3.
pub trait HasRTypeV1: HasRType<Header = RecordHeader> {}

impl<T: HasRType<Header = RecordHeader>> HasRTypeV1 for T {}
