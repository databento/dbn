//! Bit set flags used in Databento market data.

use std::fmt;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Indicates it's the last record in the event from the venue for a given
/// `instrument_id`.
pub const LAST: u8 = 1 << 7;
/// Indicates a top-of-book record, not an individual order.
pub const TOB: u8 = 1 << 6;
/// Indicates the record was sourced from a replay, such as a snapshot server.
pub const SNAPSHOT: u8 = 1 << 5;
/// Indicates an aggregated price level record, not an individual order.
pub const MBP: u8 = 1 << 4;
/// Indicates the `ts_recv` value is inaccurate due to clock issues or packet
/// reordering.
pub const BAD_TS_RECV: u8 = 1 << 3;
/// Indicates an unrecoverable gap was detected in the channel.
pub const MAYBE_BAD_BOOK: u8 = 1 << 2;

/// A transparent wrapper around the bit field used in several DBN record types,
/// namely [`MboMsg`](crate::MboMsg) and record types derived from it.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[cfg_attr(feature = "python", derive(FromPyObject), pyo3(transparent))]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
pub struct FlagSet {
    raw: u8,
}

impl fmt::Debug for FlagSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_written_flag = false;
        for (flag, name) in [
            (LAST, stringify!(LAST)),
            (TOB, stringify!(TOB)),
            (SNAPSHOT, stringify!(SNAPSHOT)),
            (MBP, stringify!(MBP)),
            (BAD_TS_RECV, stringify!(BAD_TS_RECV)),
            (MAYBE_BAD_BOOK, stringify!(MAYBE_BAD_BOOK)),
        ] {
            if (self.raw() & flag) > 0 {
                if has_written_flag {
                    write!(f, " | {name}")?;
                } else {
                    write!(f, "{name}")?;
                    has_written_flag = true;
                }
            }
        }
        if has_written_flag {
            write!(f, " ({})", self.raw())
        } else {
            write!(f, "{}", self.raw())
        }
    }
}

impl From<u8> for FlagSet {
    fn from(raw: u8) -> Self {
        Self { raw }
    }
}

impl FlagSet {
    /// Returns an empty [`FlagSet`]: one with no flags set.
    pub const fn empty() -> Self {
        Self { raw: 0 }
    }

    /// Creates a new flag set from `raw`.
    pub const fn new(raw: u8) -> Self {
        Self { raw }
    }

    /// Turns all flags off, i.e. to `false`.
    pub fn clear(&mut self) -> &mut Self {
        self.raw = 0;
        self
    }

    /// Returns the raw value.
    pub const fn raw(&self) -> u8 {
        self.raw
    }

    /// Sets the flags directly with a raw `u8`.
    pub fn set_raw(&mut self, raw: u8) {
        self.raw = raw;
    }

    /// Returns `true` if any of the flags are on or set to true.
    pub const fn any(&self) -> bool {
        self.raw > 0
    }

    /// Returns `true` if all flags are unset/false.
    pub fn is_empty(&self) -> bool {
        self.raw == 0
    }

    /// Returns `true` if it's the last record in the event from the venue for a given
    /// `instrument_id`.
    pub const fn is_last(&self) -> bool {
        (self.raw & LAST) > 0
    }

    /// Sets the `LAST` bit flag to `true` to indicate this is the last record in the
    /// event for a given instrument.
    pub fn set_last(&mut self) -> Self {
        self.raw |= LAST;
        *self
    }

    /// Returns `true` if it's a top-of-book record, not an individual order.
    pub const fn is_tob(&self) -> bool {
        (self.raw & TOB) > 0
    }

    /// Sets the `TOB` bit flag to `true` to indicate this is a top-of-book record.
    pub fn set_tob(&mut self) -> Self {
        self.raw |= TOB;
        *self
    }

    /// Returns `true` if this record was sourced from a replay, such as a snapshot
    /// server.
    pub const fn is_snapshot(&self) -> bool {
        (self.raw & SNAPSHOT) > 0
    }

    /// Sets the `SNAPSHOT` bit flag to `true` to indicate this record was sourced from
    /// a replay.
    pub fn set_snapshot(&mut self) -> Self {
        self.raw |= SNAPSHOT;
        *self
    }

    /// Returns `true` if this record is an aggregated price level record, not an
    /// individual order.
    pub const fn is_mbp(&self) -> bool {
        (self.raw & MBP) > 0
    }

    /// Sets the `MBP` bit flag to `true` to indicate this record is an aggregated price
    /// level record.
    pub fn set_mbp(&mut self) -> Self {
        self.raw |= MBP;
        *self
    }

    /// Returns `true` if this record has an inaccurate `ts_recv` value due to clock
    /// issues or packet reordering.
    pub const fn is_bad_ts_recv(&self) -> bool {
        (self.raw & BAD_TS_RECV) > 0
    }

    /// Sets the `BAD_TS_RECV` bit flag to `true` to indicate this record has an
    /// inaccurate `ts_recv` value.
    pub fn set_bad_ts_recv(&mut self) -> Self {
        self.raw |= BAD_TS_RECV;
        *self
    }

    /// Returns `true` if this record is from a channel where an unrecoverable gap was
    /// detected.
    pub const fn is_maybe_bad_book(&self) -> bool {
        (self.raw & MAYBE_BAD_BOOK) > 0
    }

    /// Sets the `MAYBE_BAD_BOOK` bit flag to `true` to indicate this record is from a
    /// channel where an unrecoverable gap was detected.
    pub fn set_maybe_bad_book(&mut self) -> Self {
        self.raw |= MAYBE_BAD_BOOK;
        *self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;

    #[rstest]
    #[case::empty(FlagSet::empty(), "0")]
    #[case::one_set(FlagSet::empty().set_mbp(), "MBP (16)")]
    #[case::three_set(FlagSet::empty().set_tob().set_snapshot().set_maybe_bad_book(), "TOB | SNAPSHOT | MAYBE_BAD_BOOK (100)")]
    #[case::reserved_set(
        FlagSet::new(255),
        "LAST | TOB | SNAPSHOT | MBP | BAD_TS_RECV | MAYBE_BAD_BOOK (255)"
    )]
    fn dbg(#[case] target: FlagSet, #[case] exp: &str) {
        assert_eq!(format!("{target:?}"), exp);
    }
}
