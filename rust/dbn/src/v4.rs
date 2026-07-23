//! Record data types for encoding different Databento [`Schema`](crate::enums::Schema)s
//! in the upcoming DBN version 4.
//!
//! In development and may change.

pub mod rtype;

use std::os::raw::c_char;

use crate::{
    record::{HasRType, Record},
    FlagSet, Publisher, RType,
};

pub mod fields;
pub mod types;

/// A non-owning immutable reference to a DBN version 4 record.
pub type RecordRef<'a> = crate::RecordRef<'a, RecordHeader>;
/// A non-owning mutable reference to a DBN version 4 record.
pub type RecordRefMut<'a> = crate::RecordRefMut<'a, RecordHeader>;

/// The length in bytes of the largest DBN version 4 record type.
///
/// Provisionally the v3 value.
pub const MAX_RECORD_LEN: usize = crate::MAX_RECORD_LEN;

/// An owned buffer holding any DBN version 4 record of a dynamic type.
pub type RecordBuf<const CAP: usize = MAX_RECORD_LEN> = crate::RecordBuf<CAP, RecordHeader>;

/// Contains common data across all DBN version 4 records including framing and type (`rtype`)
/// fields. Always found at the beginning of the record.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecordHeader {
    /// The number of bytes following the header.
    length: u16,
    /// The record type. Record types implement the trait [`HasRType`], and the
    /// [`has_rtype`][HasRType::has_rtype] function can be used to check if that type can
    /// be used to decode a message with a given rtype. The set of possible values is
    /// defined in [`rtype`].
    pub rtype: u16,
    /// The publisher ID assigned by Databento, which denotes the dataset and venue.
    ///
    /// See [Publishers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#publishers-datasets-and-venues).
    pub publisher_id: u16,
    /// Reserved for future use. Always zero.
    _reserved: [u8; 2],
    /// The numeric instrument ID. See [Instrument identifiers](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#instrument-identifiers).
    pub instrument_id: u64,
    /// The matching-engine-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_event](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-event).
    pub ts_event: u64,
}

impl RecordHeader {
    /// Creates a new `RecordHeader`. `R` and `rtype` should be compatible.
    pub const fn new<R: HasRType<Header = Self>>(
        rtype: u16,
        publisher_id: u16,
        instrument_id: u64,
        ts_event: u64,
    ) -> Self {
        Self {
            // The v4 `length` counts only the bytes following the header.
            length: (std::mem::size_of::<R>() - std::mem::size_of::<Self>()) as u16,
            rtype,
            publisher_id,
            _reserved: [0; 2],
            instrument_id,
            ts_event,
        }
    }

    /// A header whose `length` covers `record_size` total bytes, for a record whose
    /// size comes from its stream's layout rather than from a compiled struct.
    ///
    /// # Errors
    /// Returns an error if `record_size` cannot hold the header, or overflows `length`.
    pub fn for_record_size(rtype: u16, record_size: usize) -> crate::Result<Self> {
        let body = record_size
            .checked_sub(std::mem::size_of::<Self>())
            .ok_or_else(|| {
                crate::Error::encode(format!(
                    "record size {record_size} is shorter than the header"
                ))
            })?;
        Ok(Self {
            length: u16::try_from(body).map_err(|_| {
                crate::Error::encode(format!("record size {record_size} overflows `length`"))
            })?,
            rtype,
            publisher_id: 0,
            _reserved: [0; 2],
            instrument_id: 0,
            ts_event: crate::UNDEF_TIMESTAMP,
        })
    }

    /// Returns the size of the **entire** record in bytes. Unlike the v1-v3 header, the
    /// v4 `length` is a plain byte count of the fields following the header.
    pub const fn record_size(&self) -> usize {
        std::mem::size_of::<Self>() + self.length as usize
    }

    /// Tries to convert the raw record type into an enum.
    ///
    /// # Errors
    /// This function returns an error if `rtype` does not contain a valid, known
    /// [`RType`].
    pub fn rtype(&self) -> crate::Result<RType> {
        // `RType` is sized for the v1-v3 `rtype` field, so a v4 rtype beyond a byte has
        // no variant to convert to.
        u8::try_from(self.rtype)
            .ok()
            .and_then(|rtype| RType::try_from(rtype).ok())
            .ok_or_else(|| crate::Error::conversion::<RType>(format!("{:#06X}", self.rtype)))
    }

    /// Tries to convert the raw `publisher_id` into an enum.
    ///
    /// # Errors
    /// This function returns an error if `publisher_id` does not correspond with any
    /// known [`Publisher`].
    pub fn publisher(&self) -> crate::Result<Publisher> {
        Publisher::try_from(self.publisher_id)
            .map_err(|_| crate::Error::conversion::<Publisher>(self.publisher_id))
    }
}

/// A market-by-order message.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MboMsg {
    /// The common header.
    pub hd: RecordHeader,
    /// The order ID assigned at the venue.
    pub order_id: u64,
    /// The order price where every 1 unit corresponds to 1e-9, i.e. 1/1,000,000,000 or
    /// 0.000000001.
    ///
    /// See [Prices](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#prices).
    pub price: i64,
    /// The order quantity.
    pub size: u32,
    /// A bit field indicating event end, message characteristics, and data quality.
    /// See [`flags`](crate::flags) for possible values.
    pub flags: FlagSet,
    /// The channel ID assigned by Databento as an incrementing integer starting at zero.
    pub channel_id: u8,
    /// The event action. Can be **A**dd, **C**ancel, **M**odify, clea**R** book, **T**rade, **F**ill, or **N**one.
    ///
    /// See [Action](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#action).
    pub action: c_char,
    /// The side that initiates the event. Can be **A**sk for a sell order (or sell aggressor in
    /// a trade), **B**id for a buy order (or buy aggressor in a trade), or **N**one where no side is specified.
    ///
    /// See [Side](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#side).
    pub side: c_char,
    /// The capture-server-received timestamp expressed as the number of nanoseconds
    /// since the UNIX epoch.
    ///
    /// See [ts_recv](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-recv).
    pub ts_recv: u64,
    /// The matching-engine-sending timestamp expressed as the number of nanoseconds before
    /// `ts_recv`.
    ///
    /// See [ts_in_delta](https://databento.com/docs/standards-and-conventions/common-fields-enums-types#ts-in-delta).
    pub ts_in_delta: i32,
    /// The message sequence number assigned at the venue.
    pub sequence: u32,
}

impl AsRef<[u8]> for MboMsg {
    fn as_ref(&self) -> &[u8] {
        unsafe { crate::record::record_as_u8_slice(self) }
    }
}

impl Record for MboMsg {
    type Header = RecordHeader;

    fn record_size(&self) -> usize {
        self.hd.record_size()
    }

    fn rtype(&self) -> crate::Result<RType> {
        self.hd.rtype()
    }

    fn raw_rtype(&self) -> u16 {
        self.hd.rtype
    }

    fn publisher_id(&self) -> u16 {
        self.hd.publisher_id
    }

    fn publisher(&self) -> crate::Result<Publisher> {
        self.hd.publisher()
    }

    fn instrument_id(&self) -> u64 {
        self.hd.instrument_id
    }

    fn raw_ts_event(&self) -> u64 {
        self.hd.ts_event
    }

    fn raw_index_ts(&self) -> u64 {
        self.ts_recv
    }
}

impl HasRType for MboMsg {
    fn has_rtype(rtype: u16) -> bool {
        rtype == rtype::MBO
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{align_of, offset_of, size_of};

    use rstest::rstest;

    use super::*;
    use crate::{RecordRef, RecordRefMut};

    fn mbo() -> MboMsg {
        MboMsg {
            hd: RecordHeader::new::<MboMsg>(rtype::MBO, 1, 5482, 1609160400000000000),
            order_id: 16,
            price: 5500,
            size: 3,
            flags: FlagSet::empty(),
            channel_id: 14,
            action: b'B' as c_char,
            side: b'A' as c_char,
            ts_recv: 1609160400000000100,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }
    }

    #[test]
    fn header_layout() {
        assert_eq!(size_of::<RecordHeader>(), 24);
        assert_eq!(align_of::<RecordHeader>(), 8);
        assert_eq!(offset_of!(RecordHeader, rtype), 2);
        assert_eq!(offset_of!(RecordHeader, publisher_id), 4);
        assert_eq!(offset_of!(RecordHeader, instrument_id), 8);
        assert_eq!(offset_of!(RecordHeader, ts_event), 16);
    }

    // The v4 `length` counts the bytes after the header.
    #[test]
    fn new_frames_the_whole_record() {
        let hd = RecordHeader::new::<MboMsg>(rtype::MBO, 1, 5482, 0);
        assert_eq!(
            hd.length as usize,
            size_of::<MboMsg>() - size_of::<RecordHeader>()
        );
        assert_eq!(hd.record_size(), size_of::<MboMsg>());
    }

    #[rstest]
    #[case::mbo(rtype::MBO, true)]
    #[case::trades(rtype::MBP_0, false)]
    #[case::above_a_byte(0x01A0, false)]
    fn has_rtype_takes_the_full_width(#[case] rtype: u16, #[case] expected: bool) {
        assert_eq!(MboMsg::has_rtype(rtype), expected);
    }

    #[test]
    fn record_ref_round_trips() {
        let mbo = mbo();
        let rec: RecordRef<'_, RecordHeader> = RecordRef::from(&mbo);
        assert_eq!(rec.record_size(), size_of::<MboMsg>());
        assert_eq!(rec.as_ref().len(), size_of::<MboMsg>());
        assert_eq!(rec.get::<MboMsg>(), Some(&mbo));
    }

    #[test]
    fn record_ref_mut_writes_through() {
        let mut mbo = mbo();
        let mut rec: RecordRefMut<'_, RecordHeader> = RecordRefMut::from(&mut mbo);
        rec.get_mut::<MboMsg>().unwrap().order_id = 42;
        assert_eq!(mbo.order_id, 42);
    }

    #[test]
    fn record_buf_round_trips() {
        let mbo = mbo();
        let buf = RecordBuf::<{ MAX_RECORD_LEN }>::from(mbo);
        assert_eq!(buf.header().record_size(), size_of::<MboMsg>());
        assert_eq!(buf.get::<MboMsg>(), Some(&mbo));
    }
}
