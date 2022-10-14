use std::{
    io::{self, SeekFrom, Write},
    ops::Range,
};

use anyhow::{anyhow, Context};
use zstd::Encoder;

use crate::{read::SymbolMapping, Metadata};

impl Metadata {
    pub(crate) const ZSTD_MAGIC_RANGE: Range<u32> = 0x184D2A50..0x184D2A60;
    pub(crate) const SCHEMA_VERSION: u8 = 1;
    pub(crate) const VERSION_CSTR_LEN: usize = 4;
    pub(crate) const DATASET_CSTR_LEN: usize = 16;
    pub(crate) const RESERVED_LEN: usize = 39;
    pub(crate) const FIXED_METADATA_LEN: usize = 96;
    pub(crate) const SYMBOL_CSTR_LEN: usize = 22;
    pub(crate) const ZSTD_COMPRESSION_LEVEL: i32 = 0;

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
            let mut zstd_encoder =
                Encoder::new(&mut writer, Self::ZSTD_COMPRESSION_LEVEL)?.auto_finish();
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

#[cfg(test)]
mod tests {
    use std::{io::Seek, mem};

    use databento_defs::enums::{Compression, SType, Schema};

    use crate::read::{FromLittleEndianSlice, MappingInterval};

    use super::*;

    #[test]
    fn test_encode_decode_identity() {
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
}
