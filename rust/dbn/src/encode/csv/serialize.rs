use std::{ffi::c_char, io};

use csv::Writer;

use crate::{
    enums::{SecurityUpdateAction, UserDefinedInstrument},
    pretty::{fmt_px, fmt_ts},
    record::{
        c_chars_to_str, BidAskPair, ConsolidatedBidAskPair, HasRType, RecordHeader, WithTsOut,
    },
    UNDEF_PRICE, UNDEF_TIMESTAMP,
};

/// Because of the flat nature of CSVs, there are several limitations in the
/// Rust CSV serde serialization library. This trait helps work around them.
pub trait CsvSerialize {
    /// Encode the header to `csv_writer`.
    fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()>;

    /// Serialize the object to `csv_writer`. Allows custom behavior that would otherwise
    /// cause a runtime error, e.g. serializing a struct with array field.
    fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        csv_writer: &mut Writer<W>,
    ) -> csv::Result<()>;
}

impl<T: HasRType + CsvSerialize> CsvSerialize for WithTsOut<T> {
    fn serialize_header<W: io::Write>(csv_writer: &mut Writer<W>) -> csv::Result<()> {
        T::serialize_header(csv_writer)?;
        csv_writer.write_field("ts_out")
    }

    fn serialize_to<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        csv_writer: &mut Writer<W>,
    ) -> csv::Result<()> {
        self.rec
            .serialize_to::<W, PRETTY_PX, PRETTY_TS>(csv_writer)?;
        write_ts_field::<W, PRETTY_TS>(csv_writer, self.ts_out)
    }
}

pub trait WriteField {
    fn write_header<W: io::Write>(csv_writer: &mut Writer<W>, name: &str) -> csv::Result<()> {
        csv_writer.write_field(name)
    }

    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut Writer<W>,
    ) -> csv::Result<()>;
}

impl WriteField for RecordHeader {
    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut Writer<W>,
    ) -> csv::Result<()> {
        self.serialize_to::<W, PRETTY_PX, PRETTY_TS>(writer)
    }

    fn write_header<W: io::Write>(csv_writer: &mut Writer<W>, _name: &str) -> csv::Result<()> {
        Self::serialize_header(csv_writer)
    }
}

impl<const N: usize> WriteField for [BidAskPair; N] {
    fn write_header<W: io::Write>(csv_writer: &mut Writer<W>, _name: &str) -> csv::Result<()> {
        for i in 0..N {
            for f in ["bid_px", "ask_px", "bid_sz", "ask_sz", "bid_ct", "ask_ct"] {
                csv_writer.write_field(&format!("{f}_{i:02}"))?;
            }
        }
        Ok(())
    }

    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut csv::Writer<W>,
    ) -> csv::Result<()> {
        for level in self.iter() {
            write_px_field::<W, PRETTY_PX>(writer, level.bid_px)?;
            write_px_field::<W, PRETTY_PX>(writer, level.ask_px)?;
            level.bid_sz.write_field::<W, false, false>(writer)?;
            level.ask_sz.write_field::<W, false, false>(writer)?;
            level.bid_ct.write_field::<W, false, false>(writer)?;
            level.ask_ct.write_field::<W, false, false>(writer)?;
        }
        Ok(())
    }
}

impl<const N: usize> WriteField for [ConsolidatedBidAskPair; N] {
    fn write_header<W: io::Write>(csv_writer: &mut Writer<W>, _name: &str) -> csv::Result<()> {
        for i in 0..N {
            for f in ["bid_px", "ask_px", "bid_sz", "ask_sz", "bid_pb", "ask_pb"] {
                csv_writer.write_field(&format!("{f}_{i:02}"))?;
            }
        }
        Ok(())
    }

    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut csv::Writer<W>,
    ) -> csv::Result<()> {
        for level in self.iter() {
            write_px_field::<W, PRETTY_PX>(writer, level.bid_px)?;
            write_px_field::<W, PRETTY_PX>(writer, level.ask_px)?;
            level.bid_sz.write_field::<W, false, false>(writer)?;
            level.ask_sz.write_field::<W, false, false>(writer)?;
            level.bid_pb.write_field::<W, false, false>(writer)?;
            level.ask_pb.write_field::<W, false, false>(writer)?;
        }
        Ok(())
    }
}
macro_rules! impl_write_field_for {
        ($($ty:ident),+) => {
            $(
                impl WriteField for $ty {
                    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
                        &self,
                        writer: &mut Writer<W>,
                    ) -> csv::Result<()> {
                        let mut buf = itoa::Buffer::new();
                        writer.write_field(buf.format(*self))
                    }
                }
            )*
        };
    }

impl_write_field_for! {i64, u64, i32, u32, i16, u16, i8, u8}

impl WriteField for bool {
    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut Writer<W>,
    ) -> csv::Result<()> {
        writer.write_field(self.to_string())
    }
}

impl<const N: usize> WriteField for [c_char; N] {
    fn write_field<W: io::Write, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut Writer<W>,
    ) -> csv::Result<()> {
        writer.write_field(c_chars_to_str(self).unwrap_or_default())
    }
}

impl WriteField for SecurityUpdateAction {
    fn write_field<W: io::Write, const _PRETTY_PX: bool, const _PRETTY_TS: bool>(
        &self,
        writer: &mut Writer<W>,
    ) -> csv::Result<()> {
        writer.write_field([*self as u8])
    }
}

impl WriteField for UserDefinedInstrument {
    fn write_field<W: io::Write, const _PRETTY_PX: bool, const _PRETTY_TS: bool>(
        &self,
        writer: &mut Writer<W>,
    ) -> csv::Result<()> {
        writer.write_field([*self as u8])
    }
}

pub fn write_px_field<W: io::Write, const PRETTY_PX: bool>(
    csv_writer: &mut Writer<W>,
    px: i64,
) -> csv::Result<()> {
    if PRETTY_PX {
        if px == UNDEF_PRICE {
            csv_writer.write_field("")
        } else {
            csv_writer.write_field(fmt_px(px))
        }
    } else {
        csv_writer.write_field(itoa::Buffer::new().format(px))
    }
}

pub fn write_ts_field<W: io::Write, const PRETTY_TS: bool>(
    csv_writer: &mut Writer<W>,
    ts: u64,
) -> csv::Result<()> {
    if PRETTY_TS {
        match ts {
            0 | UNDEF_TIMESTAMP => csv_writer.write_field(""),
            ts => csv_writer.write_field(fmt_ts(ts)),
        }
    } else {
        csv_writer.write_field(itoa::Buffer::new().format(ts))
    }
}

pub fn write_c_char_field<W: io::Write>(csv_writer: &mut Writer<W>, c: c_char) -> csv::Result<()> {
    // Handle NUL byte
    if c == 0 {
        csv_writer.write_field([])
    } else {
        csv_writer.write_field([c as u8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_char_nul() {
        let mut buffer = Vec::new();
        let mut writer = csv::WriterBuilder::new().from_writer(&mut buffer);
        write_c_char_field(&mut writer, 0).unwrap();
        writer.write_field("a").unwrap();
        writer.flush().unwrap();
        drop(writer);
        let s = std::str::from_utf8(buffer.as_slice()).unwrap();
        assert_eq!(s, ",a");
    }
}
