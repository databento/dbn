use std::mem;

use crate::{record::as_u8_slice, HasRType, Record, RecordHeader, RecordMut, WithTsOut};

impl<T: HasRType> Record for WithTsOut<T> {
    fn header(&self) -> &RecordHeader {
        self.rec.header()
    }

    fn raw_index_ts(&self) -> u64 {
        self.rec.raw_index_ts()
    }
}

impl<T: HasRType> RecordMut for WithTsOut<T> {
    fn header_mut(&mut self) -> &mut RecordHeader {
        self.rec.header_mut()
    }
}

impl<T: HasRType> HasRType for WithTsOut<T> {
    fn has_rtype(rtype: u8) -> bool {
        T::has_rtype(rtype)
    }
}

impl<T> AsRef<[u8]> for WithTsOut<T>
where
    T: HasRType,
{
    fn as_ref(&self) -> &[u8] {
        unsafe { as_u8_slice(self) }
    }
}

impl<T: HasRType> WithTsOut<T> {
    /// Creates a new record with `ts_out`. Updates the `length` property in
    /// [`RecordHeader`] to ensure the additional field is accounted for.
    pub fn new(rec: T, ts_out: u64) -> Self {
        let mut res = Self { rec, ts_out };
        res.header_mut().length = (mem::size_of_val(&res) / RecordHeader::LENGTH_MULTIPLIER) as u8;
        res
    }

    /// Parses the raw live gateway send timestamp into a datetime.
    pub fn ts_out(&self) -> time::OffsetDateTime {
        // u64::MAX is within maximum allowable range
        time::OffsetDateTime::from_unix_timestamp_nanos(self.ts_out as i128).unwrap()
    }
}
