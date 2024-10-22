use std::ffi::c_char;

use crate::{
    json_writer::{JsonObjectWriter, NULL},
    pretty::{fmt_px, fmt_ts},
    record::{c_chars_to_str, ConsolidatedBidAskPair},
    BidAskPair, FlagSet, HasRType, Metadata, RecordHeader, SecurityUpdateAction,
    UserDefinedInstrument, WithTsOut, UNDEF_PRICE, UNDEF_TIMESTAMP,
};

/// Serializes `obj` to a JSON string.
pub fn to_json_string<T: JsonSerialize>(
    obj: &T,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
) -> String {
    let mut res = String::new();
    if should_pretty_print {
        let mut pretty = pretty_writer(&mut res);
        let mut writer = JsonObjectWriter::new(&mut pretty);
        to_json_with_writer(obj, &mut writer, use_pretty_px, use_pretty_ts);
    } else {
        let mut writer = JsonObjectWriter::new(&mut res);
        to_json_with_writer(obj, &mut writer, use_pretty_px, use_pretty_ts);
    }
    res.push('\n');
    res
}

/// Serializes `obj` to a JSON string with an optional `symbol`.
pub fn to_json_string_with_sym<T: JsonSerialize>(
    obj: &T,
    should_pretty_print: bool,
    use_pretty_px: bool,
    use_pretty_ts: bool,
    symbol: Option<&str>,
) -> String {
    let mut res = String::new();
    if should_pretty_print {
        let mut pretty = pretty_writer(&mut res);
        let mut writer = JsonObjectWriter::new(&mut pretty);
        to_json_with_writer(obj, &mut writer, use_pretty_px, use_pretty_ts);
        writer.value("symbol", symbol);
    } else {
        let mut writer = JsonObjectWriter::new(&mut res);
        to_json_with_writer(obj, &mut writer, use_pretty_px, use_pretty_ts);
        writer.value("symbol", symbol);
    }
    res.push('\n');
    res
}

fn to_json_with_writer<T: JsonSerialize, J: crate::json_writer::JsonWriter>(
    obj: &T,
    writer: &mut JsonObjectWriter<J>,
    use_pretty_px: bool,
    use_pretty_ts: bool,
) {
    match (use_pretty_px, use_pretty_ts) {
        (true, true) => obj.to_json::<J, true, true>(writer),
        (true, false) => obj.to_json::<J, true, false>(writer),
        (false, true) => obj.to_json::<J, false, true>(writer),
        (false, false) => obj.to_json::<J, false, false>(writer),
    };
}

fn pretty_writer(buffer: &mut String) -> crate::json_writer::PrettyJsonWriter<'_> {
    crate::json_writer::PrettyJsonWriter::with_indent(buffer, "    ")
}

pub trait JsonSerialize {
    fn to_json<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut JsonObjectWriter<J>,
    );
}

impl<T: HasRType + JsonSerialize> JsonSerialize for WithTsOut<T> {
    fn to_json<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut JsonObjectWriter<J>,
    ) {
        self.rec.to_json::<J, PRETTY_PX, PRETTY_TS>(writer);
        write_ts_field::<J, PRETTY_TS>(writer, "ts_out", self.ts_out);
    }
}

impl JsonSerialize for Metadata {
    fn to_json<J: crate::json_writer::JsonWriter, const _PRETTY_PX: bool, const PRETTY_TS: bool>(
        &self,
        writer: &mut JsonObjectWriter<J>,
    ) {
        writer.value("version", self.version);
        writer.value("dataset", &self.dataset);
        writer.value("schema", self.schema.map(|s| s.as_str()));
        write_ts_field::<J, PRETTY_TS>(writer, "start", self.start);
        if let Some(end) = self.end {
            write_ts_field::<J, PRETTY_TS>(writer, "end", end.get());
        } else {
            writer.value("end", NULL);
        }
        let mut buf = itoa::Buffer::new();
        writer.value("limit", self.limit.map(|l| buf.format(l.get())));
        writer.value("stype_in", self.stype_in.map(|s| s.as_str()));
        writer.value("stype_out", self.stype_out.as_str());
        writer.value("ts_out", self.ts_out);
        writer.value("symbol_cstr_len", self.symbol_cstr_len as u32);
        for (key, sym_list) in [
            ("symbols", &self.symbols),
            ("partial", &self.partial),
            ("not_found", &self.not_found),
        ] {
            let mut writer = writer.array(key);
            for symbol in sym_list {
                writer.value(symbol);
            }
        }
        let mut mappings_writer = writer.array("mappings");
        for mapping in self.mappings.iter() {
            let mut item_writer = mappings_writer.object();
            item_writer.value("raw_symbol", &mapping.raw_symbol);
            let mut interval_arr_writer = item_writer.array("intervals");
            for interval in mapping.intervals.iter() {
                let mut interval_writer = interval_arr_writer.object();
                write_date_field::<J, PRETTY_TS>(
                    &mut interval_writer,
                    "start_date",
                    &interval.start_date,
                );
                write_date_field::<J, PRETTY_TS>(
                    &mut interval_writer,
                    "end_date",
                    &interval.end_date,
                );
                interval_writer.value("symbol", &interval.symbol);
            }
        }
    }
}

pub trait WriteField {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    );
}

impl WriteField for RecordHeader {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        let mut hd_writer = writer.object(name);
        // Serialize ts_event first to be more human-readable
        write_ts_field::<J, PRETTY_TS>(&mut hd_writer, "ts_event", self.ts_event);
        hd_writer.value("rtype", self.rtype);
        hd_writer.value("publisher_id", self.publisher_id);
        hd_writer.value("instrument_id", self.instrument_id);
    }
}

impl<const N: usize> WriteField for [BidAskPair; N] {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        let mut arr_writer = writer.array(name);
        for level in self.iter() {
            let mut item_writer = arr_writer.object();
            write_px_field::<J, PRETTY_PX>(&mut item_writer, "bid_px", level.bid_px);
            write_px_field::<J, PRETTY_PX>(&mut item_writer, "ask_px", level.ask_px);
            item_writer.value("bid_sz", level.bid_sz);
            item_writer.value("ask_sz", level.ask_sz);
            item_writer.value("bid_ct", level.bid_ct);
            item_writer.value("ask_ct", level.ask_ct);
        }
    }
}

impl<const N: usize> WriteField for [ConsolidatedBidAskPair; N] {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        let mut arr_writer = writer.array(name);
        for level in self.iter() {
            let mut item_writer = arr_writer.object();
            write_px_field::<J, PRETTY_PX>(&mut item_writer, "bid_px", level.bid_px);
            write_px_field::<J, PRETTY_PX>(&mut item_writer, "ask_px", level.ask_px);
            item_writer.value("bid_sz", level.bid_sz);
            item_writer.value("ask_sz", level.ask_sz);
            item_writer.value("bid_pb", level.bid_pb);
            item_writer.value("ask_pb", level.ask_pb);
        }
    }
}

impl WriteField for FlagSet {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        writer.value(name, self.raw())
    }
}

impl WriteField for i64 {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        writer.value(name, itoa::Buffer::new().format(*self));
    }
}

impl WriteField for u64 {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        writer.value(name, itoa::Buffer::new().format(*self));
    }
}

macro_rules! impl_write_field_for {
        ($($ty:ident),+) => {
            $(
                impl WriteField for $ty {
                    fn write_field<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool, const PRETTY_TS: bool>(
                        &self,
                        writer: &mut JsonObjectWriter<J>,
                        name: &str,
                    ) {
                        writer.value(name, self);
                    }
                }
            )*
        };
    }

impl_write_field_for! {i32, u32, i16, u16, i8, u8, bool}

impl WriteField for SecurityUpdateAction {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const _PRETTY_PX: bool,
        const _PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        let mut buf = [0; 4];
        writer.value(name, &*(*self as u8 as char).encode_utf8(&mut buf));
    }
}

impl WriteField for UserDefinedInstrument {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const _PRETTY_PX: bool,
        const _PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        let mut buf = [0; 4];
        writer.value(name, &*(*self as u8 as char).encode_utf8(&mut buf));
    }
}

impl<const N: usize> WriteField for [c_char; N] {
    fn write_field<
        J: crate::json_writer::JsonWriter,
        const PRETTY_PX: bool,
        const PRETTY_TS: bool,
    >(
        &self,
        writer: &mut JsonObjectWriter<J>,
        name: &str,
    ) {
        writer.value(name, c_chars_to_str(self).unwrap_or_default());
    }
}

pub fn write_c_char_field<J: crate::json_writer::JsonWriter>(
    writer: &mut JsonObjectWriter<J>,
    name: &str,
    c: c_char,
) {
    // Handle NUL byte as null
    if c == 0 {
        writer.value(name, NULL);
    } else {
        let mut buf = [0; 4];
        let mut size = 0;
        for byte in std::ascii::escape_default(c as u8) {
            buf[size] = byte;
            size += 1;
        }
        writer.write_key(name);
        // Writing fragment to get around escaping logic since we've already escaped the string.
        // Using `std::ascii::escape_default` to be consistent between CSV and JSON.
        writer.writer.json_fragment("\"");
        writer
            .writer
            // SAFETY: [`std::ascii:escape_default`] always returns valid UTF-8
            .json_fragment(unsafe { std::str::from_utf8_unchecked(&buf[..size]) });
        writer.writer.json_fragment("\"");
    }
}

pub fn write_px_field<J: crate::json_writer::JsonWriter, const PRETTY_PX: bool>(
    writer: &mut JsonObjectWriter<J>,
    key: &str,
    px: i64,
) {
    if PRETTY_PX {
        if px == UNDEF_PRICE {
            writer.value(key, NULL);
        } else {
            writer.value(key, &fmt_px(px));
        }
    } else {
        // Convert to string to avoid a loss of precision
        writer.value(key, itoa::Buffer::new().format(px))
    }
}

pub fn write_ts_field<J: crate::json_writer::JsonWriter, const PRETTY_TS: bool>(
    writer: &mut JsonObjectWriter<J>,
    key: &str,
    ts: u64,
) {
    if PRETTY_TS {
        match ts {
            0 | UNDEF_TIMESTAMP => writer.value(key, NULL),
            ts => writer.value(key, &fmt_ts(ts)),
        };
    } else {
        // Convert to string to avoid a loss of precision
        writer.value(key, itoa::Buffer::new().format(ts));
    }
}

fn write_date_field<J: crate::json_writer::JsonWriter, const PRETTY_TS: bool>(
    writer: &mut JsonObjectWriter<J>,
    key: &str,
    date: &time::Date,
) {
    if PRETTY_TS {
        writer.value(
            key,
            &date
                .format(crate::metadata::DATE_FORMAT)
                .unwrap_or_default(),
        );
    } else {
        let mut date_int = date.year() as u32 * 10_000;
        date_int += date.month() as u32 * 100;
        date_int += date.day() as u32;
        writer.value(key, date_int);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rstest::*;

    #[rstest]
    #[case::nul(0, "null")]
    #[case::max(0xFF, r#""\xff""#)]
    #[case::reg(b'C', r#""C""#)]
    #[case::tab(b'\t', r#""\t""#)]
    #[case::newline(b'\n', r#""\n""#)]
    fn test_write_c_char_field(#[case] c: u8, #[case] exp: &str) {
        let mut buf = String::new();
        {
            let mut writer = json_writer::JSONObjectWriter::new(&mut buf);
            write_c_char_field(&mut writer, "test", c as c_char);
        }
        dbg!(&buf);
        assert_eq!(buf, format!("{{\"test\":{exp}}}"));
    }
}
