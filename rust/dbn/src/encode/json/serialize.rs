use std::ffi::c_char;

use crate::json_writer::{JsonObjectWriter, NULL};
use crate::pretty::{fmt_px, fmt_ts};
use crate::UNDEF_TIMESTAMP;
use crate::{
    enums::{SecurityUpdateAction, UserDefinedInstrument},
    record::{c_chars_to_str, BidAskPair, HasRType, RecordHeader, WithTsOut},
    Metadata, UNDEF_PRICE,
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
        writer.value("limit", self.limit.map(|l| l.to_string()).as_ref());
        writer.value("stype_in", self.stype_in.map(|s| s.as_str()));
        writer.value("stype_out", self.stype_out.as_str());
        writer.value("ts_out", self.ts_out);
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
        writer.value(name, &self.to_string());
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
        writer.value(name, &self.to_string());
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
        writer.value(name, &(*self as u8 as char).to_string());
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
        writer.value(name, &(*self as u8 as char).to_string());
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
    c_char: c_char,
) {
    if c_char == 0 {
        writer.value(name, NULL);
    } else {
        writer.value(name, &(c_char as u8 as char).to_string());
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
        writer.value(key, &px.to_string())
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
        writer.value(key, &ts.to_string());
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
