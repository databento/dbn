use std::{
    io::{self, SeekFrom, Write},
    mem,
};

use zstd::Encoder;

use crate::Metadata;

impl Metadata {
    #[allow(unused)]
    pub(crate) fn encode(&self, mut writer: impl io::Write + io::Seek) -> anyhow::Result<()> {
        const MAGIC_AND_LEN_SIZE: usize = 2 * mem::size_of::<i32>();
        writer.write_all(Self::BENTO_MAGIC.to_le_bytes().as_slice())?;
        // write placeholder frame size to filled in at the end
        writer.write_all(b"0000")?;
        writer.write_all(&[self.version])?;
        let dataset_str = self.dataset.as_str();
        writer.write_all(dataset_str.as_bytes())?;
        // pad remaining space with null bytes
        for _ in dataset_str.len()..Self::DATASET_CSTR_LEN {
            writer.write_all(&[0x0])?;
        }
        writer.write_all(&[self.schema as u8])?;
        writer.write_all(&[self.stype_in as u8])?;
        writer.write_all(&[self.stype_out as u8])?;
        writer.write_all(self.start.to_le_bytes().as_slice())?;
        writer.write_all(self.end.to_le_bytes().as_slice())?;
        writer.write_all(self.limit.to_le_bytes().as_slice())?;
        writer.write_all(&[self.encoding as u8])?;
        writer.write_all(&[self.compression as u8])?;
        writer.write_all(self.nrows.to_le_bytes().as_slice())?;
        writer.write_all(self.ncols.to_le_bytes().as_slice())?;
        let current_size = writer.stream_position()? as usize;
        // pad remaining space in fixed header with null bytes
        for _ in current_size..MAGIC_AND_LEN_SIZE + Self::FIXED_METADATA_LEN {
            writer.write_all(&[0x0])?;
        }
        {
            let mut json_buffer = Vec::with_capacity(1024);
            serde_json::to_writer(&mut json_buffer, &self.extra)?;
            let mut zstd_encoder =
                Encoder::new(&mut writer, 0 /* default compression level */)?.auto_finish();
            zstd_encoder.write_all(json_buffer.as_slice())?;
        }
        let raw_size = writer.stream_position()?;
        // go back and update the size now that we know it
        writer.seek(SeekFrom::Start(4))?;
        // magic number and size aren't included in the metadata size
        let frame_size = (raw_size - 8) as i32;
        writer.write_all(frame_size.to_le_bytes().as_slice())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use db_def::enums::{Compression, Dataset, Encoding, SType, Schema};

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
            dataset: Dataset::GlbxMdp3,
            schema: Schema::Mbp10,
            stype_in: SType::Native,
            stype_out: SType::ProductId,
            start: 1657230820000000000,
            end: 1658960170000000000,
            limit: 0,
            encoding: Encoding::Dbz,
            compression: Compression::ZStd,
            nrows: 14,
            ncols: 7,
            extra,
        };
        let mut buffer = Vec::new();
        let cursor = io::Cursor::new(&mut buffer);
        metadata.encode(cursor).unwrap();
        dbg!(&buffer);
        let res = Metadata::read(&mut &buffer[..]).unwrap();
        assert_eq!(res, metadata);
    }
}
