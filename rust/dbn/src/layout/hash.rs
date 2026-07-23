use crate::{
    layout::{FieldDef, RecordLayout, StructLayout},
    v4::fields,
};

pub(crate) fn layout_hash(struct_layouts: &[StructLayout], rec_layout: &RecordLayout) -> u64 {
    let mut crc = Crc64::new();
    for field_def in rec_layout.field_defs.iter() {
        hash_field(&mut crc, struct_layouts, field_def);
    }
    crc.finish()
}

fn hash_field(crc: &mut Crc64, struct_layouts: &[StructLayout], field_def: &FieldDef) {
    crc.hash_u16(field_def.field_id);
    crc.hash_u16(field_def.offset);
    crc.hash_u16(field_def.size);
    crc.hash_u8(field_def.type_id);
    crc.hash_u8(field_def.flags);
    // `def_index` is excluded: it's a position in a per-stream table, so hashing it
    // would give two byte-identical layouts different hashes
    crc.hash_u8(field_def.scale as u8);
    // A `def_index` past the table only reaches here from an unvalidated layout.
    if field_def.type_id == fields::STRUCT_ID {
        if let Some(struct_def) = struct_layouts.get(field_def.def_index as usize) {
            crc.hash_u16(struct_def.size);
            for sub_field_def in struct_def.field_defs.iter() {
                hash_field(crc, struct_layouts, sub_field_def);
            }
        }
    }
}

/// CRC-64-AVRO.
///
/// Reimplemented to avoid a dependency and because the implementation is small.
/// Incremental so there's no scratch buffer to allocate
#[derive(Debug)]
struct Crc64 {
    fp: u64,
}

impl Crc64 {
    const EMPTY: u64 = 0xC15D_213A_A4D7_A795;

    const TABLE: [u64; 256] = {
        let mut t = [0u64; 256];
        let mut i = 0;
        while i < 256 {
            let mut fp = i as u64;
            let mut j = 0;
            while j < 8 {
                fp = (fp >> 1) ^ (Self::EMPTY & 0u64.wrapping_sub(fp & 1)); // -(fp&1) as mask
                j += 1;
            }
            t[i] = fp;
            i += 1;
        }
        t
    };

    fn new() -> Self {
        Self { fp: Self::EMPTY }
    }

    fn hash_bytes(&mut self, b: &[u8]) {
        for &x in b {
            self.fp = (self.fp >> 8) ^ Self::TABLE[((self.fp ^ x as u64) & 0xFF) as usize];
        }
    }

    fn hash_u16(&mut self, v: u16) {
        self.hash_bytes(&v.to_le_bytes());
    }

    fn hash_u8(&mut self, v: u8) {
        self.hash_bytes(&[v]);
    }

    fn finish(self) -> u64 {
        self.fp
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{RecordLayout, StreamLayout, StructLayout};
    use rstest::*;

    // Known CRC-64-AVRO values from Apache Avro's cross-language conformance vectors
    // (share/test/data/schema-tests.txt). Each fingerprint is the CRC-64-AVRO of the
    // canonical form's UTF-8 bytes, published upstream as a signed i64; a representative
    // subset spanning primitives, unions, records, enum, fixed, array, and map is used.
    #[rstest]
    #[case::null("\"null\"", 7195948357588979594)]
    #[case::int("\"int\"", 8247732601305521295)]
    #[case::string("\"string\"", -8142146995180207161)]
    #[case::union("[\"int\",\"boolean\"]", 5392556393470105090)]
    #[case::empty_record("{\"name\":\"foo\",\"type\":\"record\",\"fields\":[]}", -4824392279771201922)]
    #[case::record("{\"name\":\"foo\",\"type\":\"record\",\"fields\":[{\"name\":\"f1\",\"type\":\"boolean\"},{\"name\":\"f2\",\"type\":\"int\"}]}", -4860222112080293046)]
    #[case::enumeration("{\"name\":\"foo\",\"type\":\"enum\",\"symbols\":[\"A1\"]}", -6342190197741309591)]
    #[case::fixed(
        "{\"name\":\"foo\",\"type\":\"fixed\",\"size\":15}",
        1756455273707447556
    )]
    #[case::array("{\"type\":\"array\",\"items\":\"null\"}", -589620603366471059)]
    #[case::map("{\"type\":\"map\",\"values\":\"string\"}", -8732877298790414990)]
    #[case::recursive("{\"name\":\"PigValue\",\"type\":\"record\",\"fields\":[{\"name\":\"value\",\"type\":[\"null\",\"int\",\"long\",\"PigValue\"]}]}", -1759257747318642341)]
    fn crc64_matches_avro_vectors(#[case] canonical: &str, #[case] fingerprint: i64) {
        let mut crc = Crc64::new();
        crc.hash_bytes(canonical.as_bytes());
        assert_eq!(crc.finish(), fingerprint as u64);
    }

    #[test]
    fn empty_input_is_seed() {
        assert_eq!(Crc64::new().finish(), Crc64::EMPTY);
    }

    #[test]
    fn hash_u16_is_little_endian() {
        let mut via_u16 = Crc64::new();
        via_u16.hash_u16(0x1234);
        let mut via_bytes = Crc64::new();
        via_bytes.hash_bytes(&0x1234u16.to_le_bytes());
        assert_eq!(via_u16.finish(), via_bytes.finish());
    }

    #[test]
    fn hash_u8_matches_single_byte() {
        let mut via_u8 = Crc64::new();
        via_u8.hash_u8(0xAB);
        let mut via_bytes = Crc64::new();
        via_bytes.hash_bytes(&[0xAB]);
        assert_eq!(via_u8.finish(), via_bytes.finish());
    }

    #[test]
    fn hashing_is_incremental() {
        let data = b"databento-dbn-v4";
        let mut whole = Crc64::new();
        whole.hash_bytes(data);
        let mut split = Crc64::new();
        split.hash_bytes(&data[..5]);
        split.hash_bytes(&data[5..9]);
        split.hash_bytes(&data[9..]);
        assert_eq!(whole.finish(), split.finish());
    }

    fn scalar_field(field_id: u16, offset: u16, size: u16, type_id: u8) -> FieldDef {
        FieldDef {
            field_id,
            offset,
            size,
            type_id,
            flags: 0,
            def_index: 0,
            scale: 0,
            _reserved: [0; 5],
        }
    }

    #[test]
    fn layout_hash_composes_primitive_over_fields_and_structs() {
        let sub = scalar_field(5, 0, 8, 0x02);
        let struct_layout = StructLayout {
            size: 8,
            name: "Sub".to_owned(),
            field_defs: vec![sub],
            field_names: vec!["x".to_owned()],
        };
        let scalar_a = scalar_field(3, 16, 2, 0x01);
        let struct_b = scalar_field(100, 24, 8, fields::STRUCT_ID);
        let record = RecordLayout {
            rtype: 1,
            schema: 0,
            base_record_size: 32,
            layout_hash: 0,
            name: "Rec".to_owned(),
            field_defs: vec![scalar_a, struct_b],
            field_names: vec!["a".to_owned(), "b".to_owned()],
        };
        let stream = StreamLayout {
            stream_layout_version: 0,
            record_layouts: vec![],
            struct_layouts: vec![struct_layout],
            label_defs: vec![],
        };

        // Regression guarding the Databento-specific field feed (which
        // fields are hashed and in what order); computed from the primitive above
        assert_eq!(
            layout_hash(&stream.struct_layouts, &record),
            0x84C3_59FB_671A_0F23
        );
    }

    #[test]
    fn layout_hash_ignores_struct_table_position() {
        let sub = StructLayout {
            size: 8,
            name: "Sub".to_owned(),
            field_defs: vec![scalar_field(5, 0, 8, 0x02)],
            field_names: vec!["x".to_owned()],
        };
        let filler = StructLayout {
            size: 4,
            name: "Filler".to_owned(),
            field_defs: vec![scalar_field(6, 0, 4, 0x03)],
            field_names: vec!["y".to_owned()],
        };
        let record_at = |def_index| {
            let mut f = scalar_field(100, 24, 8, fields::STRUCT_ID);
            f.def_index = def_index;
            RecordLayout {
                rtype: 1,
                schema: 0,
                base_record_size: 32,
                layout_hash: 0,
                name: "Rec".to_owned(),
                field_defs: vec![f],
                field_names: vec!["b".to_owned()],
            }
        };
        assert_eq!(
            layout_hash(std::slice::from_ref(&sub), &record_at(0)),
            layout_hash(&[filler, sub], &record_at(1)),
        );
    }

    /// Labels are display-only, so pointing a field at a `LabelDef` doesn't change its
    /// hash
    #[test]
    fn layout_hash_ignores_label_references() {
        let unlabeled = scalar_field(3, 24, 1, fields::ENUM_8_ID);
        let mut labeled = unlabeled;
        labeled.def_index = 7;
        let record = |f: FieldDef| RecordLayout {
            rtype: 1,
            schema: 0,
            base_record_size: 32,
            layout_hash: 0,
            name: "Rec".to_owned(),
            field_defs: vec![f],
            field_names: vec!["a".to_owned()],
        };
        assert_eq!(
            layout_hash(&[], &record(unlabeled)),
            layout_hash(&[], &record(labeled)),
        );
    }
}
