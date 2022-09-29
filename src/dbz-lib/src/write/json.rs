use std::{fmt, io};

use anyhow::Context;
use serde::Serialize;

use databento_defs::record::Record;
use serde_json::ser::{Formatter, PrettyFormatter};

use crate::Metadata;

/// Incrementally serializes the contents of `iter` into NDJSON to `writer` so the
/// contents of `iter` are not all buffered into memory at once.
pub fn write_json<F: Clone + Formatter, T>(
    mut writer: impl io::Write,
    formatter: F,
    iter: impl Iterator<Item = T>,
) -> anyhow::Result<()>
where
    T: TryFrom<Record> + Serialize + fmt::Debug,
{
    for tick in iter {
        tick.serialize(&mut serde_json::Serializer::with_formatter(
            &mut writer,
            formatter.clone(),
        ))
        .with_context(|| format!("Failed to serialize {tick:#?}"))?;
        writer.write_all(b"\n")?;
    }
    writer.flush()?;
    Ok(())
}

/// Serializes `metadata` in JSON format to `writer`.
pub fn write_json_metadata<F: Formatter>(
    mut writer: impl io::Write,
    formatter: F,
    metadata: &Metadata,
) -> anyhow::Result<()> {
    let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
    metadata.serialize(&mut serializer)?;
    // newline at EOF
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub fn pretty_formatter() -> PrettyFormatter<'static> {
    // with_indent should be `const`
    PrettyFormatter::with_indent(b"    ")
}

#[cfg(test)]
mod tests {
    use std::{io::BufWriter, os::raw::c_char};

    use super::*;
    use crate::{
        write::test_data::{BID_ASK, RECORD_HEADER},
        MappingInterval, SymbolMapping,
    };
    use databento_defs::{
        enums::{Compression, SType, Schema},
        record::{Mbp10Msg, Mbp1Msg, OhlcvMsg, StatusMsg, SymDefMsg, TickMsg, TradeMsg},
    };
    use serde_json::ser::CompactFormatter;

    fn write_json_to_string<T>(iter: impl Iterator<Item = T>, should_pretty_print: bool) -> String
    where
        T: TryFrom<Record> + Serialize + fmt::Debug,
    {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        if should_pretty_print {
            write_json(writer, pretty_formatter(), iter)
        } else {
            write_json(writer, CompactFormatter, iter)
        }
        .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    fn write_json_metadata_to_string(metadata: &Metadata, should_pretty_print: bool) -> String {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        if should_pretty_print {
            write_json_metadata(writer, pretty_formatter(), metadata)
        } else {
            write_json_metadata(writer, CompactFormatter, metadata)
        }
        .unwrap();
        String::from_utf8(buffer).expect("valid UTF-8")
    }

    const HEADER_JSON: &str =
        r#""hd":{"rtype":4,"publisher_id":1,"product_id":323,"ts_event":1658441851000000000}"#;
    const BID_ASK_JSON: &str = r#"{"bid_px":372000000000000,"ask_px":372500000000000,"bid_sz":10,"ask_sz":5,"bid_ct":5,"ask_ct":2}"#;

    #[test]
    fn test_tick_write_json() {
        let data = vec![TickMsg {
            hd: RECORD_HEADER,
            order_id: 16,
            price: 5500,
            size: 3,
            flags: -128,
            channel_id: 14,
            action: 'B' as i8,
            side: 67,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""order_id":16,"price":5500,"size":3,"flags":-128,"channel_id":14,"action":66,"side":67,"ts_recv":1658441891000000000,"ts_in_delta":22000,"sequence":1002375"#
            )
        );
    }

    #[test]
    fn test_mbo1_write_json() {
        let data = vec![Mbp1Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as i8,
            side: 67,
            flags: -128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [BID_ASK; 1],
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{},{}}}\n",
                r#""price":5500,"size":3,"action":66,"side":67,"flags":-128,"depth":9,"ts_recv":1658441891000000000,"ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"booklevel\":[{BID_ASK_JSON}]")
            )
        );
    }

    #[test]
    fn test_mbo10_write_json() {
        let data = vec![Mbp10Msg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as i8,
            side: 67,
            flags: -128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [BID_ASK; 10],
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{},{}}}\n",
                r#""price":5500,"size":3,"action":66,"side":67,"flags":-128,"depth":9,"ts_recv":1658441891000000000,"ts_in_delta":22000,"sequence":1002375"#,
                format_args!("\"booklevel\":[{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON},{BID_ASK_JSON}]")
            )
        );
    }

    #[test]
    fn test_trade_write_json() {
        let data = vec![TradeMsg {
            hd: RECORD_HEADER,
            price: 5500,
            size: 3,
            action: 'B' as i8,
            side: 67,
            flags: -128,
            depth: 9,
            ts_recv: 1658441891000000000,
            ts_in_delta: 22_000,
            sequence: 1_002_375,
            booklevel: [],
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""price":5500,"size":3,"action":66,"side":67,"flags":-128,"depth":9,"ts_recv":1658441891000000000,"ts_in_delta":22000,"sequence":1002375"#,
            )
        );
    }

    #[test]
    fn test_ohlcv_write_json() {
        let data = vec![OhlcvMsg {
            hd: RECORD_HEADER,
            open: 5000,
            high: 8000,
            low: 3000,
            close: 6000,
            volume: 55_000,
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""open":5000,"high":8000,"low":3000,"close":6000,"volume":55000"#,
            )
        );
    }

    #[test]
    fn test_status_write_json() {
        let mut group = [0; 21];
        for (i, c) in "group".chars().enumerate() {
            group[i] = c as c_char;
        }
        let data = vec![StatusMsg {
            hd: RECORD_HEADER,
            ts_recv: 1658441891000000000,
            group,
            trading_status: 3,
            halt_reason: 4,
            trading_event: 6,
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                r#""ts_recv":1658441891000000000,"group":"group","trading_status":3,"halt_reason":4,"trading_event":6"#,
            )
        );
    }

    #[test]
    fn test_symdef_write_json() {
        let data = vec![SymDefMsg {
            hd: RECORD_HEADER,
            ts_recv: 1658441891000000000,
            min_price_increment: 100,
            display_factor: 1000,
            expiration: 1698450000000000000,
            activation: 1697350000000000000,
            high_limit_price: 1_000_000,
            low_limit_price: -1_000_000,
            max_price_variation: 0,
            trading_reference_price: 500_000,
            unit_of_measure_qty: 5,
            min_price_increment_amount: 5,
            price_ratio: 10,
            inst_attrib_value: 10,
            underlying_id: 256785,
            cleared_volume: 0,
            market_depth_implied: 0,
            market_depth: 13,
            market_segment_id: 0,
            max_trade_vol: 10_000,
            min_lot_size: 1,
            min_lot_size_block: 1000,
            min_lot_size_round_lot: 100,
            min_trade_vol: 1,
            open_interest_qty: 0,
            contract_multiplier: 0,
            decay_quantity: 0,
            original_contract_size: 0,
            related_security_id: 0,
            trading_reference_date: 0,
            appl_id: 0,
            maturity_month_year: 0,
            decay_start_date: 0,
            chan: 4,
            currency: [0; 4],
            settl_currency: ['U' as c_char, 'S' as c_char, 'D' as c_char, 0],
            secsubtype: [0; 6],
            symbol: [0; 22],
            group: [0; 21],
            exchange: [0; 5],
            asset: [0; 7],
            cfi: [0; 7],
            security_type: [0; 7],
            unit_of_measure: [0; 31],
            underlying: [0; 21],
            related: [0; 21],
            match_algorithm: 1,
            md_security_trading_status: 2,
            main_fraction: 4,
            price_display_format: 8,
            settl_price_type: 9,
            sub_fraction: 23,
            underlying_product: 10,
            security_update_action: 7,
            maturity_month_month: 8,
            maturity_month_day: 9,
            maturity_month_week: 11,
            user_defined_instrument: 1,
            contract_multiplier_unit: 0,
            flow_schedule_type: 5,
            tick_rule: 0,
            _dummy: [0; 3],
        }];
        let res = write_json_to_string(data.into_iter(), false);

        assert_eq!(
            res,
            format!(
                "{{{HEADER_JSON},{}}}\n",
                concat!(
                    r#""ts_recv":1658441891000000000,"min_price_increment":100,"display_factor":1000,"expiration":1698450000000000000,"activation":1697350000000000000,"#,
                    r#""high_limit_price":1000000,"low_limit_price":-1000000,"max_price_variation":0,"trading_reference_price":500000,"unit_of_measure_qty":5,"#,
                    r#""min_price_increment_amount":5,"price_ratio":10,"inst_attrib_value":10,"underlying_id":256785,"cleared_volume":0,"market_depth_implied":0,"#,
                    r#""market_depth":13,"market_segment_id":0,"max_trade_vol":10000,"min_lot_size":1,"min_lot_size_block":1000,"min_lot_size_round_lot":100,"min_trade_vol":1,"#,
                    r#""open_interest_qty":0,"contract_multiplier":0,"decay_quantity":0,"original_contract_size":0,"related_security_id":0,"trading_reference_date":0,"appl_id":0,"#,
                    r#""maturity_month_year":0,"decay_start_date":0,"chan":4,"currency":"","settl_currency":"USD","secsubtype":"","symbol":"","group":"","exchange":"","asset":"","cfi":"","#,
                    r#""security_type":"","unit_of_measure":"","underlying":"","related":"","match_algorithm":1,"md_security_trading_status":2,"main_fraction":4,"price_display_format":8,"#,
                    r#""settl_price_type":9,"sub_fraction":23,"underlying_product":10,"security_update_action":7,"maturity_month_month":8,"maturity_month_day":9,"maturity_month_week":11,"#,
                    r#""user_defined_instrument":1,"contract_multiplier_unit":0,"flow_schedule_type":5,"tick_rule":0"#
                )
            )
        );
    }

    #[test]
    fn test_metadata_write_json() {
        let metadata = Metadata {
            version: 1,
            dataset: "GLBX.MDP3".to_owned(),
            schema: Schema::Ohlcv1h,
            start: 1662734705128748281,
            end: 1662734720914876944,
            limit: 0,
            record_count: 3,
            compression: Compression::ZStd,
            stype_in: SType::ProductId,
            stype_out: SType::Native,
            symbols: vec!["ESZ2".to_owned()],
            partial: Vec::new(),
            not_found: Vec::new(),
            mappings: vec![SymbolMapping {
                native: "ESZ2".to_owned(),
                intervals: vec![MappingInterval {
                    start_date: time::Date::from_calendar_date(2022, time::Month::September, 9)
                        .unwrap(),
                    end_date: time::Date::from_calendar_date(2022, time::Month::September, 10)
                        .unwrap(),
                    symbol: "ESH2".to_owned(),
                }],
            }],
        };
        let res = write_json_metadata_to_string(&metadata, false);
        assert_eq!(
            res,
            "{\"version\":1,\"dataset\":\"GLBX.MDP3\",\"schema\":\"ohlcv-1h\",\"start\"\
            :1662734705128748281,\"end\":1662734720914876944,\"limit\":0,\"record_count\":3,\"\
            compression\":\"zstd\",\"stype_in\":\"product_id\",\"stype_out\":\"native\",\"symbols\"\
            :[\"ESZ2\"],\"partial\":[],\"not_found\":[],\"mappings\":[{\"native\":\"ESZ2\",\
            \"intervals\":[{\"start_date\":\"2022-09-09\",\"end_date\":\"2022-09-10\",\"symbol\":\
            \"ESH2\"}]}]}\n"
        );
    }
}
