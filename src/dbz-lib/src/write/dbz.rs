use std::{
    io::{self, SeekFrom, Write},
    mem,
    ops::Range,
    slice,
};

use anyhow::{anyhow, Context};
use databento_defs::record::ConstTypeId;
use streaming_iterator::StreamingIterator;
use zstd::{stream::AutoFinishEncoder, Encoder};

use crate::{read::SymbolMapping, Metadata};

pub(crate) const SCHEMA_VERSION: u8 = 1;

/// Create a new Zstd encoder with default settings
fn new_encoder<'a, W: io::Write>(writer: W) -> anyhow::Result<AutoFinishEncoder<'a, W>> {
    pub(crate) const ZSTD_COMPRESSION_LEVEL: i32 = 0;

    let mut encoder = Encoder::new(writer, ZSTD_COMPRESSION_LEVEL)?;
    encoder.include_checksum(true)?;
    Ok(encoder.auto_finish())
}

impl Metadata {
    pub(crate) const ZSTD_MAGIC_RANGE: Range<u32> = 0x184D2A50..0x184D2A60;
    pub(crate) const VERSION_CSTR_LEN: usize = 4;
    pub(crate) const DATASET_CSTR_LEN: usize = 16;
    pub(crate) const RESERVED_LEN: usize = 39;
    pub(crate) const FIXED_METADATA_LEN: usize = 96;
    pub(crate) const SYMBOL_CSTR_LEN: usize = 22;

    pub fn encode(&self, mut writer: impl io::Write + io::Seek) -> anyhow::Result<()> {
        writer.write_all(Self::ZSTD_MAGIC_RANGE.start.to_le_bytes().as_slice())?;
        // write placeholder frame size to filled in at the end
        writer.write_all(b"0000")?;
        writer.write_all(b"DBZ")?;
        writer.write_all(&[self.version])?;
        Self::encode_fixed_len_cstr::<_, { Self::DATASET_CSTR_LEN }>(&mut writer, &self.dataset)?;
        writer.write_all((self.schema as u16).to_le_bytes().as_slice())?;
        Self::encode_range_and_counts(
            &mut writer,
            self.start,
            self.end,
            self.limit,
            self.record_count,
        )?;
        writer.write_all(&[self.compression as u8])?;
        writer.write_all(&[self.stype_in as u8])?;
        writer.write_all(&[self.stype_out as u8])?;
        // padding
        writer.write_all(&[0; Self::RESERVED_LEN])?;
        {
            // remaining metadata is compressed
            let mut zstd_encoder = new_encoder(&mut writer)?;
            // schema_definition_length
            zstd_encoder.write_all(0u32.to_le_bytes().as_slice())?;

            Self::encode_repeated_symbol_cstr(&mut zstd_encoder, self.symbols.as_slice())
                .with_context(|| "Failed to encode symbols")?;
            Self::encode_repeated_symbol_cstr(&mut zstd_encoder, self.partial.as_slice())
                .with_context(|| "Failed to encode partial")?;
            Self::encode_repeated_symbol_cstr(&mut zstd_encoder, self.not_found.as_slice())
                .with_context(|| "Failed to encode not_found")?;
            Self::encode_symbol_mappings(&mut zstd_encoder, self.mappings.as_slice())?;
        }

        let raw_size = writer.stream_position()?;
        // go back and update the size now that we know it
        writer.seek(SeekFrom::Start(4))?;
        // magic number and size aren't included in the metadata size
        let frame_size = (raw_size - 8) as u32;
        writer.write_all(frame_size.to_le_bytes().as_slice())?;
        // go back to end to leave `writer` in a place for more data to be written
        writer.seek(SeekFrom::End(0))?;

        Ok(())
    }

    pub fn update_encoded(
        mut writer: impl io::Write + io::Seek,
        start: u64,
        end: u64,
        limit: u64,
        record_count: u64,
    ) -> anyhow::Result<()> {
        /// Byte position of the field `start`
        const START_SEEK_FROM: SeekFrom =
            SeekFrom::Start((8 + 4 + Metadata::DATASET_CSTR_LEN + 2) as u64);

        writer
            .seek(START_SEEK_FROM)
            .with_context(|| "Failed to seek to write position".to_owned())?;
        Self::encode_range_and_counts(&mut writer, start, end, limit, record_count)?;
        writer
            .seek(SeekFrom::End(0))
            .with_context(|| "Failed to seek back to end".to_owned())?;
        Ok(())
    }

    fn encode_range_and_counts(
        writer: &mut impl io::Write,
        start: u64,
        end: u64,
        limit: u64,
        record_count: u64,
    ) -> anyhow::Result<()> {
        writer.write_all(start.to_le_bytes().as_slice())?;
        writer.write_all(end.to_le_bytes().as_slice())?;
        writer.write_all(limit.to_le_bytes().as_slice())?;
        writer.write_all(record_count.to_le_bytes().as_slice())?;
        Ok(())
    }

    fn encode_repeated_symbol_cstr(
        writer: &mut impl io::Write,
        symbols: &[String],
    ) -> anyhow::Result<()> {
        writer.write_all((symbols.len() as u32).to_le_bytes().as_slice())?;
        for symbol in symbols {
            Self::encode_fixed_len_cstr::<_, { Self::SYMBOL_CSTR_LEN }>(writer, symbol)?;
        }

        Ok(())
    }

    fn encode_symbol_mappings(
        writer: &mut impl io::Write,
        symbol_mappings: &[SymbolMapping],
    ) -> anyhow::Result<()> {
        // encode mappings_count
        writer.write_all((symbol_mappings.len() as u32).to_le_bytes().as_slice())?;
        for symbol_mapping in symbol_mappings {
            Self::encode_symbol_mapping(writer, symbol_mapping)?;
        }
        Ok(())
    }

    fn encode_symbol_mapping(
        writer: &mut impl io::Write,
        symbol_mapping: &SymbolMapping,
    ) -> anyhow::Result<()> {
        Self::encode_fixed_len_cstr::<_, { Self::SYMBOL_CSTR_LEN }>(
            writer,
            &symbol_mapping.native,
        )?;
        // encode interval_count
        writer.write_all(
            (symbol_mapping.intervals.len() as u32)
                .to_le_bytes()
                .as_slice(),
        )?;
        for interval in symbol_mapping.intervals.iter() {
            Self::encode_date(writer, interval.start_date)?;
            Self::encode_date(writer, interval.end_date)?;
            Self::encode_fixed_len_cstr::<_, { Self::SYMBOL_CSTR_LEN }>(writer, &interval.symbol)?;
        }
        Ok(())
    }

    // Can't specify const generic with impl trait until Rust 1.63, see
    // https://github.com/rust-lang/rust/issues/83701
    fn encode_fixed_len_cstr<W: io::Write, const LEN: usize>(
        writer: &mut W,
        string: &str,
    ) -> anyhow::Result<()> {
        if !string.is_ascii() {
            return Err(anyhow!(
                "'{string}' can't be encoded in DBZ because it contains non-ASCII characters"
            ));
        }
        if string.len() > LEN {
            return Err(anyhow!(
                "'{string}' is too long to be encoded in DBZ; it cannot be longer {LEN} characters"
            ));
        }
        writer.write_all(string.as_bytes())?;
        // pad remaining space with null bytes
        for _ in string.len()..LEN {
            writer.write_all(&[0])?;
        }
        Ok(())
    }

    fn encode_date(writer: &mut impl io::Write, date: time::Date) -> anyhow::Result<()> {
        let mut date_int = date.year() as u32 * 10_000;
        date_int += date.month() as u32 * 100;
        date_int += date.day() as u32;
        writer.write_all(date_int.to_le_bytes().as_slice())?;
        Ok(())
    }
}

unsafe fn as_u8_slice<T: Sized>(data: &T) -> &[u8] {
    slice::from_raw_parts(data as *const T as *const u8, mem::size_of::<T>())
}

/// Incrementally serializes the records in `iter` in the DBZ format to `writer`.
pub fn write_dbz_stream<T>(
    writer: impl io::Write,
    mut stream: impl StreamingIterator<Item = T>,
) -> anyhow::Result<()>
where
    T: ConstTypeId + Sized,
{
    let mut encoder = new_encoder(writer)
        .with_context(|| "Failed to create Zstd encoder for writing DBZ".to_owned())?;
    while let Some(record) = stream.next() {
        let bytes = unsafe {
            // Safety: all records, types implementing `ConstTypeId` are POD
            as_u8_slice(record)
        };
        match encoder.write_all(bytes) {
            // closed pipe, should stop writing output
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
            r => r,
        }
        .with_context(|| "Failed to serialize {record:#?}")?;
    }
    encoder.flush()?;
    Ok(())
}

/// Incrementally serializes the records in `iter` in the DBZ format to `writer`.
pub fn write_dbz<'a, T>(
    writer: impl io::Write,
    iter: impl Iterator<Item = &'a T>,
) -> anyhow::Result<()>
where
    T: 'a + ConstTypeId + Sized,
{
    let mut encoder = new_encoder(writer)
        .with_context(|| "Failed to create Zstd encoder for writing DBZ".to_owned())?;
    for record in iter {
        let bytes = unsafe {
            // Safety: all records, types implementing `ConstTypeId` are POD
            as_u8_slice(record)
        };
        match encoder.write_all(bytes) {
            // closed pipe, should stop writing output
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => return Ok(()),
            r => r,
        }
        .with_context(|| "Failed to serialize {record:#?}")?;
    }
    encoder.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::c_char,
        fmt,
        io::{BufWriter, Seek},
        mem,
    };

    use databento_defs::{
        enums::{Compression, SType, Schema},
        record::{Mbp1Msg, OhlcvMsg, RecordHeader, StatusMsg, TickMsg, TradeMsg},
    };

    use crate::{
        read::{FromLittleEndianSlice, MappingInterval},
        write::test_data::{VecStream, BID_ASK, RECORD_HEADER},
        DbzStreamIter,
    };

    use super::*;

    #[test]
    fn test_encode_decode_metadata_identity() {
        let mut extra = serde_json::Map::default();
        extra.insert(
            "Key".to_owned(),
            serde_json::Value::Number(serde_json::Number::from_f64(4.0).unwrap()),
        );
        let metadata = Metadata {
            version: 1,
            dataset: "GLBX.MDP3".to_owned(),
            schema: Schema::Mbp10,
            stype_in: SType::Native,
            stype_out: SType::ProductId,
            start: 1657230820000000000,
            end: 1658960170000000000,
            limit: 0,
            compression: Compression::ZStd,
            record_count: 14,
            symbols: vec!["ES".to_owned(), "NG".to_owned()],
            partial: vec!["ESM2".to_owned()],
            not_found: vec!["QQQQQ".to_owned()],
            mappings: vec![
                SymbolMapping {
                    native: "ES.0".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date: time::Date::from_calendar_date(2022, time::Month::July, 26)
                            .unwrap(),
                        end_date: time::Date::from_calendar_date(2022, time::Month::September, 1)
                            .unwrap(),
                        symbol: "ESU2".to_owned(),
                    }],
                },
                SymbolMapping {
                    native: "NG.0".to_owned(),
                    intervals: vec![
                        MappingInterval {
                            start_date: time::Date::from_calendar_date(2022, time::Month::July, 26)
                                .unwrap(),
                            end_date: time::Date::from_calendar_date(2022, time::Month::August, 29)
                                .unwrap(),
                            symbol: "NGU2".to_owned(),
                        },
                        MappingInterval {
                            start_date: time::Date::from_calendar_date(
                                2022,
                                time::Month::August,
                                29,
                            )
                            .unwrap(),
                            end_date: time::Date::from_calendar_date(
                                2022,
                                time::Month::September,
                                1,
                            )
                            .unwrap(),
                            symbol: "NGV2".to_owned(),
                        },
                    ],
                },
            ],
        };
        let mut buffer = Vec::new();
        let cursor = io::Cursor::new(&mut buffer);
        metadata.encode(cursor).unwrap();
        dbg!(&buffer);
        let res = Metadata::read(&mut &buffer[..]).unwrap();
        dbg!(&res, &metadata);
        assert_eq!(res, metadata);
    }

    #[test]
    fn test_encode_repeated_symbol_cstr() {
        let mut buffer = Vec::new();
        let symbols = vec![
            "NG".to_owned(),
            "HP".to_owned(),
            "HPQ".to_owned(),
            "LNQ".to_owned(),
        ];
        Metadata::encode_repeated_symbol_cstr(&mut buffer, symbols.as_slice()).unwrap();
        assert_eq!(
            buffer.len(),
            mem::size_of::<u32>() + symbols.len() * Metadata::SYMBOL_CSTR_LEN
        );
        assert_eq!(u32::from_le_slice(&buffer[..4]), 4);
        for (i, symbol) in symbols.iter().enumerate() {
            let offset = i * Metadata::SYMBOL_CSTR_LEN;
            assert_eq!(
                &buffer[4 + offset..4 + offset + symbol.len()],
                symbol.as_bytes()
            );
        }
    }

    #[test]
    fn test_encode_fixed_len_cstr() {
        let mut buffer = Vec::new();
        Metadata::encode_fixed_len_cstr::<_, { Metadata::SYMBOL_CSTR_LEN }>(&mut buffer, "NG")
            .unwrap();
        assert_eq!(buffer.len(), Metadata::SYMBOL_CSTR_LEN);
        assert_eq!(&buffer[..2], b"NG");
        for b in buffer[2..].iter() {
            assert_eq!(*b, 0);
        }
    }

    #[test]
    fn test_encode_date() {
        let date = time::Date::from_calendar_date(2020, time::Month::May, 17).unwrap();
        let mut buffer = Vec::new();
        Metadata::encode_date(&mut buffer, date).unwrap();
        assert_eq!(buffer.len(), mem::size_of::<u32>());
        assert_eq!(buffer.as_slice(), 20200517u32.to_le_bytes().as_slice());
    }

    #[test]
    fn test_update_encoded() {
        let orig_metadata = Metadata {
            version: 1,
            dataset: "GLBX.MDP3".to_owned(),
            schema: Schema::Mbo,
            stype_in: SType::Smart,
            stype_out: SType::Native,
            start: 1657230820000000000,
            end: 1658960170000000000,
            limit: 0,
            record_count: 1_450_000,
            compression: Compression::ZStd,
            symbols: vec![],
            partial: vec![],
            not_found: vec![],
            mappings: vec![],
        };
        let mut buffer = Vec::new();
        let cursor = io::Cursor::new(&mut buffer);
        orig_metadata.encode(cursor).unwrap();
        let orig_res = Metadata::read(&mut &buffer[..]).unwrap();
        assert_eq!(orig_metadata, orig_res);
        let mut cursor = io::Cursor::new(&mut buffer);
        assert_eq!(cursor.position(), 0);
        cursor.seek(SeekFrom::End(0)).unwrap();
        let before_pos = cursor.position();
        assert!(before_pos != 0);
        let new_start = 1697240529000000000;
        let new_end = 17058980170000000000;
        let new_limit = 10;
        let new_record_count = 100_678;
        Metadata::update_encoded(&mut cursor, new_start, new_end, new_limit, new_record_count)
            .unwrap();
        assert_eq!(before_pos, cursor.position());
        let res = Metadata::read(&mut &buffer[..]).unwrap();
        assert!(res != orig_res);
        assert_eq!(res.start, new_start);
        assert_eq!(res.end, new_end);
        assert_eq!(res.limit, new_limit);
        assert_eq!(res.record_count, new_record_count);
    }

    fn encode_records_and_stub_metadata<T>(schema: Schema, records: Vec<T>) -> (Vec<u8>, Metadata)
    where
        T: ConstTypeId + Clone,
    {
        let mut buffer = Vec::new();
        let writer = BufWriter::new(&mut buffer);
        write_dbz_stream(writer, VecStream::new(records.clone())).unwrap();
        dbg!(&buffer);
        let metadata = Metadata {
            version: 1,
            dataset: "GLBX.MDP3".to_owned(),
            schema,
            start: 0,
            end: 0,
            limit: 0,
            record_count: records.len() as u64,
            compression: Compression::None,
            stype_in: SType::Native,
            stype_out: SType::ProductId,
            symbols: vec![],
            partial: vec![],
            not_found: vec![],
            mappings: vec![],
        };
        (buffer, metadata)
    }

    fn assert_encode_decode_record_identity<T>(schema: Schema, records: Vec<T>)
    where
        T: ConstTypeId + Clone + fmt::Debug + PartialEq,
    {
        let (buffer, metadata) = encode_records_and_stub_metadata(schema, records.clone());
        let mut iter: DbzStreamIter<&[u8], T> =
            DbzStreamIter::new(buffer.as_slice(), metadata).unwrap();
        let mut res = Vec::new();
        while let Some(rec) = iter.next() {
            res.push(rec.to_owned());
        }
        dbg!(&res, &records);
        assert_eq!(res, records);
    }

    #[test]
    fn test_encode_decode_mbo_identity() {
        let records = vec![
            TickMsg {
                hd: RecordHeader {
                    rtype: TickMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                order_id: 2,
                price: 9250000000,
                size: 25,
                flags: -128,
                channel_id: 1,
                action: 'B' as i8,
                side: 67,
                ts_recv: 1658441891000000000,
                ts_in_delta: 1000,
                sequence: 98,
            },
            TickMsg {
                hd: RecordHeader {
                    rtype: TickMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                order_id: 3,
                price: 9350000000,
                size: 800,
                flags: 0,
                channel_id: 1,
                action: 'C' as i8,
                side: 67,
                ts_recv: 1658441991000000000,
                ts_in_delta: 750,
                sequence: 101,
            },
        ];
        assert_encode_decode_record_identity(Schema::Mbo, records);
    }

    #[test]
    fn test_encode_decode_mbp1_identity() {
        let records = vec![
            Mbp1Msg {
                hd: RecordHeader {
                    rtype: Mbp1Msg::TYPE_ID,
                    ..RECORD_HEADER
                },
                price: 925000000000,
                size: 300,
                action: 'S' as i8,
                side: 67,
                flags: -128,
                depth: 1,
                ts_recv: 1658442001000000000,
                ts_in_delta: 750,
                sequence: 100,
                booklevel: [BID_ASK; 1],
            },
            Mbp1Msg {
                hd: RecordHeader {
                    rtype: Mbp1Msg::TYPE_ID,
                    ..RECORD_HEADER
                },
                price: 925000000000,
                size: 50,
                action: 'B' as i8,
                side: 67,
                flags: -128,
                depth: 1,
                ts_recv: 1658542001000000000,
                ts_in_delta: 787,
                sequence: 101,
                booklevel: [BID_ASK; 1],
            },
        ];
        assert_encode_decode_record_identity(Schema::Mbp1, records);
    }

    #[test]
    fn test_encode_decode_trade_identity() {
        let records = vec![
            TradeMsg {
                hd: RecordHeader {
                    rtype: TradeMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                price: 925000000000,
                size: 1,
                action: 'T' as i8,
                side: 'B' as i8,
                flags: 0,
                depth: 4,
                ts_recv: 1658441891000000000,
                ts_in_delta: 234,
                sequence: 1005,
                booklevel: [],
            },
            TradeMsg {
                hd: RecordHeader {
                    rtype: TradeMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                price: 925000000000,
                size: 10,
                action: 'T' as i8,
                side: 'S' as i8,
                flags: 0,
                depth: 1,
                ts_recv: 1659441891000000000,
                ts_in_delta: 10358,
                sequence: 1010,
                booklevel: [],
            },
        ];
        assert_encode_decode_record_identity(Schema::Trades, records);
    }

    #[test]
    fn test_encode_decode_ohlcv_identity() {
        let records = vec![
            OhlcvMsg {
                hd: RecordHeader {
                    rtype: OhlcvMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                open: 92500000000,
                high: 95200000000,
                low: 91200000000,
                close: 91600000000,
                volume: 6785,
            },
            OhlcvMsg {
                hd: RecordHeader {
                    rtype: OhlcvMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                open: 91600000000,
                high: 95100000000,
                low: 91600000000,
                close: 92300000000,
                volume: 7685,
            },
        ];
        assert_encode_decode_record_identity(Schema::Ohlcv1D, records);
    }

    #[test]
    fn test_encode_decode_status_identity() {
        let mut group = [0; 21];
        for (i, c) in "group".chars().enumerate() {
            group[i] = c as c_char;
        }
        let records = vec![
            StatusMsg {
                hd: RecordHeader {
                    rtype: StatusMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                ts_recv: 1658441891000000000,
                group,
                trading_status: 3,
                halt_reason: 4,
                trading_event: 5,
            },
            StatusMsg {
                hd: RecordHeader {
                    rtype: StatusMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                ts_recv: 1658541891000000000,
                group,
                trading_status: 4,
                halt_reason: 5,
                trading_event: 6,
            },
        ];
        assert_encode_decode_record_identity(Schema::Status, records);
    }

    #[test]
    fn test_decode_malformed_encoded_dbz() {
        let records = vec![
            OhlcvMsg {
                hd: RecordHeader {
                    rtype: OhlcvMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                open: 92500000000,
                high: 95200000000,
                low: 91200000000,
                close: 91600000000,
                volume: 6785,
            },
            OhlcvMsg {
                hd: RecordHeader {
                    rtype: OhlcvMsg::TYPE_ID,
                    ..RECORD_HEADER
                },
                open: 91600000000,
                high: 95100000000,
                low: 91600000000,
                close: 92300000000,
                volume: 7685,
            },
        ];
        let wrong_schema = Schema::Mbo;
        let (buffer, metadata) = encode_records_and_stub_metadata(wrong_schema, records);
        type WrongRecord = TickMsg;
        let mut iter: DbzStreamIter<&[u8], WrongRecord> =
            DbzStreamIter::new(buffer.as_slice(), metadata).unwrap();
        // check doesn't panic
        assert!(iter.next().is_none());
        assert!(iter.next().is_none());
    }
}
