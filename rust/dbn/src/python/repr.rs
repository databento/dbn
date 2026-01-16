//! Python-specific `__repr__` implementation support.
//!
//! This module provides traits and helpers for generating Python-style string
//! representations of DBN records. Unlike Rust's `Debug` trait, Python repr:
//! - Flattens nested structs (e.g. `RecordHeader`) to match how they appear in
//!   Python
//! - Uses Python enum syntax: `EnumName.VARIANT` instead of `EnumName::Variant`

use std::fmt::{self, Write};
use std::os::raw::c_char;

use crate::pretty;
use crate::record::c_chars_to_str;
use crate::{BidAskPair, ConsolidatedBidAskPair, FlagSet, RecordHeader};

/// Trait for Python-specific `__repr__` output on record types.
pub trait WritePyRepr {
    /// Whether this type's fields should be flattened into the parent repr.
    const SHOULD_FLATTEN: bool = false;
    /// Writes a Python-style string representation to `s`.
    ///
    /// # Errors
    /// This function returns an error if it fails to expand the buffer to fit
    /// the string.
    fn write_py_repr(&self, s: &mut String) -> fmt::Result;
}

macro_rules! impl_write_py_repr_debug {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl WritePyRepr for $ty {
                fn write_py_repr(&self, s: &mut String) -> fmt::Result {
                    write!(s, "{self:?}")
                }
            }
        )+
    };
}

impl_write_py_repr_debug! {
    i64, u64, i32, u32, i16, u16, i8, u8, bool,
    FlagSet,
}

impl WritePyRepr for RecordHeader {
    const SHOULD_FLATTEN: bool = true;

    fn write_py_repr(&self, s: &mut String) -> fmt::Result {
        write!(s, "rtype=")?;
        match self.rtype() {
            Ok(rtype) => rtype.write_py_repr(s)?,
            Err(_) => write!(s, "{}", self.rtype)?,
        }
        write!(s, ", publisher_id=")?;
        match self.publisher() {
            Ok(p) => p.write_py_repr(s)?,
            Err(_) => write!(s, "{}", self.publisher_id)?,
        }
        write!(s, ", instrument_id={}, ", self.instrument_id)?;
        fmt_ts(s, "ts_event", self.ts_event)
    }
}

impl<const N: usize> WritePyRepr for [c_char; N] {
    fn write_py_repr(&self, s: &mut String) -> fmt::Result {
        match c_chars_to_str(self) {
            Ok(v) => write!(s, "'{v}'"),
            Err(_) => write!(s, "{self:?}"),
        }
    }
}

impl WritePyRepr for &str {
    fn write_py_repr(&self, s: &mut String) -> fmt::Result {
        write!(s, "'{self}'")
    }
}

impl<const N: usize> WritePyRepr for [BidAskPair; N] {
    const SHOULD_FLATTEN: bool = true;

    fn write_py_repr(&self, s: &mut String) -> fmt::Result {
        // Flatten array with indexed field names, including both raw and pretty prices
        for (i, level) in self.iter().enumerate() {
            if i > 0 {
                write!(s, ", ")?;
            }
            // bid_px raw then pretty
            write!(
                s,
                "bid_px_{i:02}={}, pretty_bid_px_{i:02}={}, ask_px_{i:02}={}, pretty_ask_px_{i:02}={}, bid_sz_{i:02}={}, ask_sz_{i:02}={}, bid_ct_{i:02}={}, ask_ct_{i:02}={}",
                level.bid_px,
                pretty::px_to_f64(level.bid_px),
                level.ask_px,
                pretty::px_to_f64(level.ask_px),
                level.bid_sz,
                level.ask_sz,
                level.bid_ct,
                level.ask_ct
            )?;
        }
        Ok(())
    }
}

impl<const N: usize> WritePyRepr for [ConsolidatedBidAskPair; N] {
    const SHOULD_FLATTEN: bool = true;

    fn write_py_repr(&self, s: &mut String) -> fmt::Result {
        // Flatten array with indexed field names, including both raw and pretty prices
        for (i, level) in self.iter().enumerate() {
            if i > 0 {
                write!(s, ", ")?;
            }
            write!(
                s,
                "bid_px_{i:02}={}, pretty_bid_px_{i:02}={}, ask_px_{i:02}={}, pretty_ask_px_{i:02}={}, bid_sz_{i:02}={}, ask_sz_{i:02}={}, bid_pb_{i:02}={}, ask_pb_{i:02}={}",
                level.bid_px,
                pretty::px_to_f64(level.bid_px),
                level.ask_px,
                pretty::px_to_f64(level.ask_px),
                level.bid_sz,
                level.ask_sz,
                level.bid_pb,
                level.ask_pb
            )?;
        }
        Ok(())
    }
}

/// Formats a fixed-precision price field for a Python repr.
///
/// # Errors
/// This function returns an error if it fails to expand the buffer to fit
/// the string.
pub fn fmt_px(s: &mut String, field_name: &str, px: i64) -> fmt::Result {
    write!(s, "{field_name}={px}, ")?;
    write!(s, "pretty_{field_name}={}", pretty::px_to_f64(px))
}

/// Formats a nanosecond UNIX timestamp field for a Python repr.
///
/// # Errors
/// This function returns an error if it fails to expand the buffer to fit
/// the string.
pub fn fmt_ts(s: &mut String, field_name: &str, ts: u64) -> fmt::Result {
    write!(s, "{field_name}={ts}, ")?;
    write!(s, "pretty_{field_name}='{}'", pretty::fmt_ts(ts))
}

/// Format a `c_char` field that should be displayed as a Python enum.
/// Falls back to char representation if parsing fails.
///
/// # Errors
/// This function returns an error if it fails to expand the buffer to fit
/// the string.
pub fn fmt_c_char_enum<E, F>(
    f: &mut String,
    field_name: &str,
    raw: c_char,
    parser: F,
) -> fmt::Result
where
    E: WritePyRepr,
    F: FnOnce() -> crate::Result<E>,
{
    write!(f, "{field_name}=")?;
    match parser() {
        Ok(e) => e.write_py_repr(f),
        Err(_) => write!(f, "'{}'", raw as u8 as char),
    }
}

/// Format an enum value obtained via a method call.
///
/// # Errors
/// This function returns an error if it fails to expand the buffer to fit
/// the string.
pub fn fmt_enum_method<E, F>(f: &mut String, field_name: &str, getter: F) -> fmt::Result
where
    E: WritePyRepr,
    F: FnOnce() -> crate::Result<E>,
{
    write!(f, "{field_name}=")?;
    match getter() {
        Ok(e) => e.write_py_repr(f),
        Err(_) => write!(f, "None"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CbboMsg, Mbp10Msg, Mbp1Msg, SType, StatMsg, SymbolMappingMsg, UNDEF_PRICE};

    #[test]
    fn test_fmt_px() {
        let mut s = String::new();
        fmt_px(&mut s, "price", 150_250_000_000).unwrap();
        assert_eq!(s, "price=150250000000, pretty_price=150.25");
    }

    #[test]
    fn test_fmt_px_undef() {
        let mut s = String::new();
        fmt_px(&mut s, "price", UNDEF_PRICE).unwrap();
        assert_eq!(s, "price=9223372036854775807, pretty_price=NaN");
    }

    #[test]
    fn test_flags_empty() {
        let mut s = String::new();
        FlagSet::empty().write_py_repr(&mut s).unwrap();
        assert_eq!(s, "0");
    }

    #[test]
    fn test_flags_set() {
        let mut s = String::new();
        let flags = FlagSet::empty().set_last().set_snapshot();
        flags.write_py_repr(&mut s).unwrap();
        assert_eq!(s, "LAST | SNAPSHOT (160)");
    }

    #[test]
    fn test_bid_ask_pair_array_flattens() {
        let levels = [BidAskPair {
            bid_px: 100_250_000_000,
            ask_px: 101_000_000_000,
            bid_sz: 10,
            ask_sz: 20,
            bid_ct: 5,
            ask_ct: 8,
        }];
        let mut s = String::new();
        levels.write_py_repr(&mut s).unwrap();
        assert_eq!(
            s,
            r"bid_px_00=100250000000, pretty_bid_px_00=100.25, ask_px_00=101000000000, pretty_ask_px_00=101, bid_sz_00=10, ask_sz_00=20, bid_ct_00=5, ask_ct_00=8"
        );
    }

    #[test]
    fn test_mbp1_msg_repr() {
        let msg = Mbp1Msg {
            hd: RecordHeader::new::<Mbp1Msg>(crate::rtype::MBP_1, 1, 12345, 1_000_000_000),
            price: 150_250_000_000,
            size: 100,
            action: b'A' as i8,
            side: b'B' as i8,
            flags: FlagSet::empty().set_last(),
            depth: 0,
            ts_recv: 1_000_000_100,
            ts_in_delta: 100,
            sequence: 1,
            levels: [BidAskPair {
                bid_px: 150_000_000_000,
                ask_px: 150_500_000_000,
                bid_sz: 50,
                ask_sz: 75,
                bid_ct: 3,
                ask_ct: 4,
            }],
        };
        let mut s = String::new();
        msg.write_py_repr(&mut s).unwrap();
        assert_eq!(
            s,
            r"Mbp1Msg(ts_recv=1000000100, pretty_ts_recv='1970-01-01T00:00:01.000000100Z', rtype=<RType.MBP_1: 1>, publisher_id=GLBX.MDP3.GLBX (1), instrument_id=12345, ts_event=1000000000, pretty_ts_event='1970-01-01T00:00:01.000000000Z', action='A', side='B', depth=0, price=150250000000, pretty_price=150.25, size=100, flags=LAST (128), ts_in_delta=100, sequence=1, bid_px_00=150000000000, pretty_bid_px_00=150, ask_px_00=150500000000, pretty_ask_px_00=150.5, bid_sz_00=50, ask_sz_00=75, bid_ct_00=3, ask_ct_00=4)"
        );
    }

    #[test]
    fn test_symbol_mapping_msg_repr() {
        let msg = SymbolMappingMsg::new(
            12345,
            1_704_067_200_000_000_000, // 2024-01-01 00:00Z
            SType::RawSymbol,
            "AAPL",
            SType::InstrumentId,
            "AAPL.XNAS",
            1_704_067_200_000_000_000,
            1_704_153_600_000_000_000, // 2024-01-02 00:00Z
        )
        .unwrap();
        let mut s = String::new();
        msg.write_py_repr(&mut s).unwrap();
        // Check key parts of the repr
        assert!(s.starts_with("SymbolMappingMsg("));
        assert!(s.contains("stype_in_symbol='AAPL'"));
        assert!(s.contains("stype_out_symbol='AAPL.XNAS'"));
        assert!(s.contains("start_ts="));
        assert!(s.contains("pretty_start_ts="));
        assert!(s.contains("end_ts="));
        assert!(s.contains("pretty_end_ts="));
    }

    #[test]
    fn test_stat_msg_repr() {
        let stat = StatMsg {
            hd: RecordHeader::new::<StatMsg>(crate::rtype::STATISTICS, 1, 12345, 1_000_000_000),
            ts_recv: 1_000_000_100,
            ts_ref: 1_000_000_000,
            price: 150_250_000_000,
            quantity: 1000,
            sequence: 42,
            ts_in_delta: 100,
            stat_type: 1,
            channel_id: 0,
            update_action: 1,
            stat_flags: 0,
            ..Default::default()
        };
        let mut s = String::new();
        stat.write_py_repr(&mut s).unwrap();
        assert!(s.starts_with("StatMsg("));
        assert!(s.contains("price=150250000000, pretty_price=150.25"));
        assert!(s.contains("ts_recv="));
        assert!(s.contains("pretty_ts_recv="));
        assert!(s.contains("ts_ref="));
        assert!(s.contains("pretty_ts_ref="));
    }

    #[test]
    fn test_cbbo_msg_repr() {
        let cbbo = CbboMsg {
            hd: RecordHeader::new::<CbboMsg>(crate::rtype::CBBO_1S, 1, 12345, 1_000_000_000),
            price: 150_250_000_000,
            size: 100,
            side: b'B' as i8,
            flags: FlagSet::empty().set_last(),
            ts_recv: 1_000_000_100,
            levels: [ConsolidatedBidAskPair {
                bid_px: 150_000_000_000,
                ask_px: 150_500_000_000,
                bid_sz: 50,
                ask_sz: 75,
                bid_pb: 1,
                ask_pb: 2,
                ..Default::default()
            }],
            ..CbboMsg::default_for_schema(crate::Schema::Cbbo1S)
        };
        let mut s = String::new();
        cbbo.write_py_repr(&mut s).unwrap();
        assert!(s.starts_with("CbboMsg("));
        assert!(s.contains("side='B'"));
        assert!(s.contains("bid_px_00="));
        assert!(s.contains("pretty_bid_px_00="));
        assert!(s.contains("bid_pb_00="));
        assert!(s.contains("ask_pb_00="));
        // Hidden fields should not appear
        assert!(!s.contains("_reserved"));
    }

    #[test]
    fn test_mbp10_msg_repr() {
        let mut mbp = Mbp10Msg {
            hd: RecordHeader::new::<Mbp10Msg>(crate::rtype::MBP_10, 1, 12345, 1_000_000_000),
            price: 150_250_000_000,
            size: 100,
            action: b'A' as i8,
            side: b'B' as i8,
            flags: FlagSet::empty().set_last(),
            depth: 0,
            ts_recv: 1_000_000_100,
            ts_in_delta: 100,
            sequence: 1,
            ..Default::default()
        };
        mbp.levels[0] = BidAskPair {
            bid_px: 150_000_000_000,
            ask_px: 150_500_000_000,
            bid_sz: 50,
            ask_sz: 75,
            bid_ct: 3,
            ask_ct: 4,
        };
        mbp.levels[1] = BidAskPair {
            bid_px: 149_750_000_000,
            ask_px: 150_750_000_000,
            bid_sz: 100,
            ask_sz: 120,
            bid_ct: 5,
            ask_ct: 6,
        };
        let mut s = String::new();
        mbp.write_py_repr(&mut s).unwrap();
        assert!(s.starts_with("Mbp10Msg("));
        assert!(s.contains("bid_px_00=150000000000"));
        assert!(s.contains("pretty_bid_px_00=150"));
        assert!(s.contains("bid_px_01=149750000000"));
        assert!(s.contains("pretty_bid_px_01=149.75"));
        assert!(s.contains("bid_px_09="));
        assert!(s.contains("ask_ct_09="));
    }
}
