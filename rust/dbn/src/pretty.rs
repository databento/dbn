//! Contains new types for pretty-printing timestamp and prices found in DBN records.

use std::fmt;

use time::format_description::BorrowedFormatItem;

use crate::FIXED_PRICE_SCALE;

/// A [new type](https://doc.rust-lang.org/rust-by-example/generics/new_types.html)
/// for formatting nanosecond UNIX timestamps to the canonical ISO 8601 format used
/// by Databento.
///
/// Supports
/// - width `{:N}` to specify a minimum width of `N` characters
/// - fill and alignment: change the default fill character from a space
///   and alignment from the default of right-aligned. See the
///   [format docs](https://doc.rust-lang.org/std/fmt/index.html#fillalignment) for
///   details
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Ts(pub u64);

/// A [new type](https://doc.rust-lang.org/rust-by-example/generics/new_types.html) for
/// formatting the fixed-precision prices used in DBN.
///
/// Supports
/// - sign `{:+}` to always print the sign
/// - width `{:N}` to specify a minimum width of `N` characters
/// - fill and alignment: change the default fill character from a space
///   and alignment from the default of right-aligned. See the
///   [format docs](https://doc.rust-lang.org/std/fmt/index.html#fillalignment) for
///   details
/// - precision `{:.N}` to print `N` decimal places. By default all 9 are printed
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Px(pub i64);

impl From<u64> for Ts {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<i64> for Px {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl fmt::Debug for Ts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for Px {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self, f)
    }
}

impl fmt::Display for Ts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const TS_FORMAT: &[BorrowedFormatItem<'static>] = time::macros::format_description!(
            "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9]Z"
        );
        let ts = self.0;
        if ts != 0 {
            // Should always be in range because we're widening from u64 to i128
            let dt = time::OffsetDateTime::from_unix_timestamp_nanos(ts as i128).unwrap();
            if let Ok(dt_str) = dt.format(TS_FORMAT) {
                f.pad(&dt_str)?;
            } else {
                // Fall back to regular int formatting
                fmt::Display::fmt(&ts, f)?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Px {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const DIVISORS: [i64; 9] = [
            0,
            100_000_000,
            10_000_000,
            1_000_000,
            100_000,
            10_000,
            1_000,
            100,
            10,
        ];
        let px = self.0;
        if px == crate::UNDEF_PRICE {
            f.write_str("UNDEF_PRICE")
        } else {
            let (is_nonnegative, px_abs) = if px < 0 { (false, -px) } else { (true, px) };
            let px_integer = px_abs / FIXED_PRICE_SCALE;
            let px_fraction = px_abs % FIXED_PRICE_SCALE;
            match f.precision() {
                Some(0) => f.pad_integral(is_nonnegative, "", itoa::Buffer::new().format(px_abs)),
                Some(precision @ ..9) => f.pad_integral(
                    is_nonnegative,
                    "",
                    &format!(
                        "{px_integer}.{:0precision$}",
                        px_fraction / DIVISORS[precision]
                    ),
                ),
                Some(_) | None => f.pad_integral(
                    is_nonnegative,
                    "",
                    &format!("{px_integer}.{px_fraction:09}"),
                ),
            }
        }
    }
}

/// Converts a fixed-precision price to a decimal string with all 9 decimal places
/// printed. Use [`Px`] to customize the number of printed decimal places, alignment,
/// fill, and other formatting options.
pub fn fmt_px(px: i64) -> String {
    let mut out = String::new();
    fmt_px_into(&mut out, px)
        // Writing to a string is infallible
        .unwrap();
    out
}

pub(crate) fn fmt_px_into<W: fmt::Write>(mut out: W, px: i64) -> fmt::Result {
    if px == crate::UNDEF_PRICE {
        write!(out, "UNDEF_PRICE")
    } else {
        let (sign, px_abs) = if px < 0 { ("-", -px) } else { ("", px) };
        let px_integer = px_abs / FIXED_PRICE_SCALE;
        let px_fraction = px_abs % FIXED_PRICE_SCALE;
        write!(
            out,
            "{sign}{}.{:0>9}",
            itoa::Buffer::new().format(px_integer),
            itoa::Buffer::new().format(px_fraction)
        )
    }
}

/// Converts a nanosecond UNIX timestamp to a human-readable string in the format
/// `[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:9]Z`.
///
/// Note: this function does not check for [`UNDEF_TIMESTAMP`](crate::UNDEF_TIMESTAMP).
pub fn fmt_ts(ts: u64) -> String {
    format!("{}", Ts(ts))
}

/// Converts a fixed-precision price to a floating point.
///
/// `UNDEF_PRICE` will be converted to NaN.
pub fn px_to_f64(px: i64) -> f64 {
    if px == crate::UNDEF_PRICE {
        f64::NAN
    } else {
        px as f64 / FIXED_PRICE_SCALE as f64
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use crate::{UNDEF_PRICE, UNDEF_TIMESTAMP};

    use super::*;

    #[rstest]
    #[case::negative(-100_000, "-0.000100000")]
    #[case::positive(32_500_000_000, "32.500000000")]
    #[case::leading_zero(101_005_000_000, "101.005000000")]
    #[case::zero(0, "0.000000000")]
    #[case::undef(UNDEF_PRICE, "UNDEF_PRICE")]
    fn test_pretty_px(#[case] num: i64, #[case] exp: &str) {
        assert_eq!(fmt_px(num), exp);
        assert_eq!(format!("{}", Px(num)), exp);
    }

    #[rstest]
    #[case::negative(-100_000, "-0.000100000")]
    #[case::positive(300_000, "+0.000300000")]
    #[case::positive(32_500_000_000, "+32.500000000")]
    #[case::zero(0, "+0.000000000")]
    fn test_sign(#[case] num: i64, #[case] exp: &str) {
        let num = Px(num);
        assert_eq!(format!("{num:+}"), exp);
    }

    #[rstest]
    #[case::positive(32_500_000_000, 3, "32.500")]
    #[case::leading_zero(101_005_000_000, 5, "101.00500")]
    #[case::leading_zero(75_000_000, 6, "0.075000")]
    #[case::trunc(32_123_456_789, 2, "32.12")]
    fn test_precision(#[case] num: i64, #[case] precision: usize, #[case] exp: &str) {
        let num = Px(num);
        assert_eq!(format!("{num:.precision$}"), exp);
    }

    #[rstest]
    #[case::positive(32_500_000_000, 4, 3, "32.500", "32.500", "32.500")]
    #[case::positive(32_500_000_000, 8, 3, "  32.500", " 32.500 ", "32.500  ")]
    // Center-aligned defaults to trailing padding when there's an odd number of padding spaces
    #[case::leading_zero(101_005_000_000, 10, 5, " 101.00500", "101.00500 ", "101.00500 ")]
    #[case::leading_zero(75_000_000, 13, 6, "     0.075000", "  0.075000   ", "0.075000     ")]
    #[case::trunc(32_123_456_789, 7, 2, "  32.12", " 32.12 ", "32.12  ")]
    #[case::trunc(
        32_123_456_789,
        16,
        8,
        "     32.12345678",
        "  32.12345678   ",
        "32.12345678     "
    )]
    fn test_alignment_width_precision(
        #[case] num: i64,
        #[case] width: usize,
        #[case] precision: usize,
        #[case] exp_right: &str,
        #[case] exp_center: &str,
        #[case] exp_left: &str,
    ) {
        let num = Px(num);
        assert_eq!(format!("{num:width$.precision$}"), exp_right);
        assert_eq!(format!("{num:>width$.precision$}"), exp_right);
        assert_eq!(format!("{num:^width$.precision$}"), exp_center);
        assert_eq!(format!("{num:<width$.precision$}"), exp_left);
    }

    #[rstest]
    #[case::signed_zero(format!("{:+}", 0), "+0")]
    #[case::uneven_center_align(format!("{:^5}", 12), " 12  ")]
    #[case::high_width_truncating_precision(format!("{0:0<8.2}", 0.125), "0.120000")]
    fn confirm_std_behavior(#[case] out: String, #[case] exp: &str) {
        assert_eq!(out, exp);
    }

    #[rstest]
    #[case::positive(32_500_000_000, 4, 3, "32.500", "32.500", "32.500")]
    #[case::positive(32_500_000_000, 8, 3, "0032.500", "032.5000", "32.50000")]
    // Center-aligned defaults to trailing padding when there's an odd number of padding spaces
    #[case::leading_zero(101_005_000_000, 10, 5, "0101.00500", "101.005000", "101.005000")]
    #[case::leading_zero(75_000_000, 13, 6, "000000.075000", "000.075000000", "0.07500000000")]
    #[case::trunc(32_123_456_789, 7, 2, "0032.12", "032.120", "32.1200")]
    #[case::trunc(
        32_123_456_789,
        16,
        8,
        "0000032.12345678",
        "0032.12345678000",
        "32.1234567800000"
    )]
    fn test_zero_fill_alignment_width_precision(
        #[case] num: i64,
        #[case] width: usize,
        #[case] precision: usize,
        #[case] exp_right: &str,
        #[case] exp_center: &str,
        #[case] exp_left: &str,
    ) {
        let num = Px(num);
        assert_eq!(format!("{num:0width$.precision$}"), exp_right);
        assert_eq!(format!("{num:0>width$.precision$}"), exp_right);
        assert_eq!(format!("{num:0^width$.precision$}"), exp_center);
        assert_eq!(format!("{num:0<width$.precision$}"), exp_left);
    }

    #[rstest]
    #[case::zero(0, "")]
    #[case::one(1, "1970-01-01T00:00:00.000000001Z")]
    #[case::recent(1622838300000000000, "2021-06-04T20:25:00.000000000Z")]
    #[case::max(UNDEF_TIMESTAMP - 1, "2554-07-21T23:34:33.709551614Z")]
    fn test_fmt_ts(#[case] ts: u64, #[case] exp: &str) {
        assert_eq!(fmt_ts(ts), exp);
    }

    #[test]
    fn test_ts_alignment_and_fill() {
        let ts = 1622838300000000000;
        assert_eq!(
            format!("{:33}", Ts(ts)),
            "2021-06-04T20:25:00.000000000Z   "
        );
        assert_eq!(
            format!("{:<33}", Ts(ts)),
            "2021-06-04T20:25:00.000000000Z   "
        );
        assert_eq!(
            format!("{:^33}", Ts(ts)),
            " 2021-06-04T20:25:00.000000000Z  "
        );
        assert_eq!(
            format!("{:>33}", Ts(ts)),
            "   2021-06-04T20:25:00.000000000Z"
        );
    }
}
