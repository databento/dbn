use crate::{
    layout::{
        FieldDef, LabelDef, RecordLayout, RecordLayoutDesc, StreamLayout, StructLayout,
        StructLayoutDesc,
    },
    v4::{
        fields::{self, Field, FieldType, FixedWidth},
        types::{CStr, Decimal, Symbol},
        RecordHeader,
    },
    RType,
};

use super::{hash::layout_hash, validate::validate};

/// Assembles a [`FieldDef`] through named setters so its many same-width fields
/// can't be transposed. The [`FieldDef`] struct literal lives only in
/// [`build`](FieldDefBuilder::build).
pub(crate) struct FieldDefBuilder {
    field_id: u16,
    type_id: u8,
    offset: u16,
    size: u16,
    flags: u8,
    def_index: u16,
    scale: i8,
}

impl FieldDefBuilder {
    pub(super) const fn new(field_id: u16, type_id: u8) -> Self {
        Self {
            field_id,
            type_id,
            offset: 0,
            size: 0,
            flags: 0,
            def_index: 0,
            scale: 0,
        }
    }

    pub(super) const fn offset(mut self, offset: u16) -> Self {
        self.offset = offset;
        self
    }

    pub(super) const fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    pub(super) const fn flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    pub(super) const fn def_index(mut self, def_index: u16) -> Self {
        self.def_index = def_index;
        self
    }

    pub(super) const fn scale(mut self, scale: i8) -> Self {
        self.scale = scale;
        self
    }

    /// The only place the [`FieldDef`] struct literal appears.
    pub(super) const fn build(self) -> FieldDef {
        FieldDef {
            field_id: self.field_id,
            offset: self.offset,
            size: self.size,
            type_id: self.type_id,
            flags: self.flags,
            def_index: self.def_index,
            scale: self.scale,
            _reserved: [0; 5],
        }
    }

    /// A padding field of `size` bytes at `offset`. Explicit so the field table covers
    /// the record with no gaps.
    const fn padding(offset: u16, size: u16) -> FieldDef {
        Self::new(fields::PADDING, fields::PADDING_ID)
            .offset(offset)
            .size(size)
            .build()
    }
}

/// Assembles a [`StreamLayout`] computing offsets and padding.
pub struct StreamLayoutBuilder {
    version: u16,
    symbol_cstr_len: u16,
    struct_layouts: Vec<StructLayout>,
    label_defs: Vec<LabelDef>,
    records: Vec<RecordLayout>,
}

/// Places the fields of one [`RecordLayout`] after its header. Created by
/// [`StreamLayoutBuilder::record`].
pub struct RecordLayoutBuilder<'a> {
    parent: &'a mut StreamLayoutBuilder,
    rtype: u16,
    schema: u16,
    name: String,
    seq: FieldCursor,
}

/// Places the fields of one [`StructLayout`]. Created by
/// [`StreamLayoutBuilder::struct_layout`].
pub struct StructLayoutBuilder<'a> {
    parent: &'a mut StreamLayoutBuilder,
    name: String,
    seq: FieldCursor,
}

impl StreamLayoutBuilder {
    /// Creates a builder for a stream with layout revision `version` whose symbol fields
    /// are `symbol_cstr_len` bytes wide.
    pub fn new(version: u16, symbol_cstr_len: u16) -> Self {
        Self {
            version,
            symbol_cstr_len,
            struct_layouts: vec![record_header_struct()],
            label_defs: vec![],
            records: vec![],
        }
    }

    /// Appends a [`LabelDef`] and returns its `label_defs` index.
    pub fn label(&mut self, def: LabelDef) -> u16 {
        self.label_defs.push(def);
        u16::try_from(self.label_defs.len() - 1).expect("exceeded maximum number of label defs")
    }

    /// Opens a record layout for `rtype`, starting with the header.
    pub fn record(&mut self, rtype: u16, schema: u16, name: &str) -> RecordLayoutBuilder<'_> {
        RecordLayoutBuilder::new(self, rtype, schema, name)
    }

    /// Adds the layout of the compiled record struct `R` for `rtype`.
    pub fn record_of<R: RecordLayoutDesc>(&mut self, rtype: u16) -> &mut Self {
        let schema = RType::try_into_schema(rtype).map_or(crate::NULL_SCHEMA, |s| s as u16);
        let mut rec = self.record(rtype, schema, R::NAME);
        R::add_fields(&mut rec);
        for &(f, name) in R::REQUIRED_FIELDS {
            rec.symbol_field(f, name);
        }
        rec.finish();
        self
    }

    /// Opens a struct layout for a struct nested in records.
    pub fn struct_layout(&mut self, name: &str) -> StructLayoutBuilder<'_> {
        StructLayoutBuilder::new(self, name)
    }

    /// The `struct_layouts` index of `S`, describing it on first use.
    fn struct_index_of<S: StructLayoutDesc>(&mut self) -> u16 {
        if let Some(idx) = self.struct_layouts.iter().position(|s| s.name == S::NAME) {
            return u16::try_from(idx).expect("exceeded maximum number of struct layouts");
        }
        let mut s = self.struct_layout(S::NAME);
        S::add_fields(&mut s);
        s.finish()
    }

    /// Validates the assembled layout and stamps each record's `layout_hash`.
    ///
    /// # Errors
    /// Returns an error if the layout violates any of the stream layout invariants.
    pub fn build(self) -> crate::Result<StreamLayout> {
        let mut layout = StreamLayout {
            stream_layout_version: self.version,
            record_layouts: self.records,
            struct_layouts: self.struct_layouts,
            label_defs: self.label_defs,
        };
        validate(&layout, self.symbol_cstr_len)?;
        let hashes: Vec<u64> = layout
            .record_layouts
            .iter()
            .map(|rec| layout_hash(&layout.struct_layouts, rec))
            .collect();
        for (rec, hash) in layout.record_layouts.iter_mut().zip(hashes) {
            rec.layout_hash = hash;
        }
        Ok(layout)
    }
}

impl<'a> RecordLayoutBuilder<'a> {
    fn new(parent: &'a mut StreamLayoutBuilder, rtype: u16, schema: u16, name: &str) -> Self {
        let mut rec = Self {
            parent,
            rtype,
            schema,
            name: name.to_owned(),
            seq: FieldCursor::empty(),
        };
        // The header is one `STRUCT_ID` field referencing the shared
        // `RecordHeader` layout at `struct_layouts[0]`
        rec.struct_field(fields::HD.id(), "hd", HEADER_STRUCT_IDX);
        rec
    }

    /// Moves the cursor to `offset`, filling any gap with padding.
    ///
    /// # Panics
    /// Panics if `offset` is behind the cursor, i.e. it overlaps a placed field.
    pub fn at(&mut self, offset: usize) -> &mut Self {
        self.seq.at(offset);
        self
    }

    /// Places a fixed-width field at the cursor, after any alignment padding.
    pub fn field<T: FixedWidth>(&mut self, f: Field<T>, name: &str) -> &mut Self {
        self.seq.scalar(f, name);
        self
    }

    /// Places a C string field `width` bytes wide at the cursor.
    pub fn str_field(&mut self, f: Field<CStr>, name: &str, width: u16) -> &mut Self {
        self.seq.string(f, name, width);
        self
    }

    /// Places a symbol field of the stream's symbol width at the cursor.
    pub fn symbol_field(&mut self, f: Field<Symbol>, name: &str) -> &mut Self {
        self.seq.string(f, name, self.parent.symbol_cstr_len);
        self
    }

    /// Places a nested struct field referencing `struct_layouts[struct_idx]`.
    pub fn struct_field(&mut self, field_id: u16, name: &str, struct_idx: u16) -> &mut Self {
        let sd = &self.parent.struct_layouts[struct_idx as usize];
        let nested = Nested {
            struct_idx,
            size: sd.size,
            align: sd.align(),
            flags: 0,
        };
        self.seq.nested(field_id, name, nested);
        self
    }

    /// A fixed-count array of the struct `S`, e.g. `Mbp10Msg`'s `[BidAskPair; 10]` book
    /// levels, adding `S`'s layout to the stream if absent. `size` spans all `N`
    /// elements; the array flag lets a reader recover the count as `size / struct.size`.
    pub fn struct_array_of<S: StructLayoutDesc, const N: usize>(
        &mut self,
        f: Field<[S; N]>,
        name: &str,
    ) -> &mut Self
    where
        [S; N]: FieldType,
    {
        let struct_idx = self.parent.struct_index_of::<S>();
        let count = u16::try_from(N).expect("struct array count exceeds u16");
        let sd = &self.parent.struct_layouts[struct_idx as usize];
        let nested = Nested {
            struct_idx,
            size: sd
                .size
                .checked_mul(count)
                .expect("struct array size exceeds u16"),
            align: sd.align(),
            flags: fields::FLAG_IS_ARRAY,
        };
        self.seq.nested(f.id(), name, nested);
        self
    }

    /// Places a fixed-width extension field past `base_record_size`. Declare every base
    /// field first.
    pub fn ext_field<T: FixedWidth>(&mut self, f: Field<T>, name: &str) -> &mut Self {
        self.seq.seal_base();
        self.seq.scalar(f, name);
        self
    }

    /// Pads the base and any extension tail to 8-byte boundaries and appends the record
    /// to the stream.
    pub fn finish(self) {
        let mut seq = self.seq;
        let base = seq.finish_record();
        self.parent.records.push(RecordLayout {
            rtype: self.rtype,
            schema: self.schema,
            base_record_size: base,
            // filled by StreamLayoutBuilder::build()
            layout_hash: 0,
            name: self.name,
            field_defs: seq.field_defs,
            field_names: seq.field_names,
        });
    }
}

impl<'a> StructLayoutBuilder<'a> {
    fn new(parent: &'a mut StreamLayoutBuilder, name: &str) -> Self {
        Self {
            parent,
            name: name.to_owned(),
            seq: FieldCursor::empty(),
        }
    }

    /// Moves the cursor to `offset`, filling any gap with padding.
    ///
    /// # Panics
    /// Panics if `offset` is behind the cursor, i.e. it overlaps a placed field.
    pub fn at(&mut self, offset: usize) -> &mut Self {
        self.seq.at(offset);
        self
    }

    /// Places a fixed-width field at the cursor, after any alignment padding.
    pub fn field<T: FixedWidth>(&mut self, f: Field<T>, name: &str) -> &mut Self {
        self.seq.scalar(f, name);
        self
    }

    /// Appends the struct to the stream and returns its `struct_layouts` index.
    pub fn finish(self) -> u16 {
        let mut seq = self.seq;
        let struct_size = seq.finish_base();
        self.parent.struct_layouts.push(StructLayout {
            size: struct_size,
            name: self.name,
            field_defs: seq.field_defs,
            field_names: seq.field_names,
        });
        u16::try_from(self.parent.struct_layouts.len() - 1)
            .expect("exceeded maximum number of struct layouts")
    }
}

/// The shared `RecordHeader` layout is always first.
const HEADER_STRUCT_IDX: u16 = 0;

/// The uniform 24-byte record header, built through the cursor like any struct so
/// there's a single field-construction path. `length` is unnamed (not presented).
fn record_header_struct() -> StructLayout {
    let mut seq = FieldCursor::empty();
    // Length is hidden from CSV and JSON: no encoded name
    seq.scalar(fields::LENGTH, "");
    seq.scalar(fields::RTYPE, "rtype");
    seq.scalar(fields::PUBLISHER_ID, "publisher_id");
    seq.scalar(fields::INSTRUMENT_ID, "instrument_id");
    seq.scalar(fields::TS_EVENT, "ts_event");
    let struct_size = seq.finish_base();
    debug_assert_eq!(struct_size as usize, std::mem::size_of::<RecordHeader>());
    StructLayout {
        size: struct_size,
        name: "RecordHeader".to_owned(),
        field_defs: seq.field_defs,
        field_names: seq.field_names,
    }
}

/// The geometry of a nested-struct field, resolved from the referenced
/// [`StructLayout`] before placement.
struct Nested {
    struct_idx: u16,
    /// Spans every element when `flags` marks an array.
    size: u16,
    align: u16,
    flags: u8,
}

/// A field-placement cursor that computes offsets and padding.
#[derive(Default)]
struct FieldCursor {
    field_defs: Vec<FieldDef>,
    field_names: Vec<String>,
    cursor: u16,
    base: Option<u16>,
}

impl FieldCursor {
    /// A cursor for a nested struct body, starting at offset 0.
    fn empty() -> Self {
        Self::default()
    }

    fn scalar<T: FixedWidth>(&mut self, f: Field<T>, name: &str) {
        // TODO(cg): unlabeled for now until label defs are wired up
        let def_index = if fields::uses_label_def(T::TYPE_ID) {
            FieldDef::UNDEF_LABEL_DEF
        } else {
            0
        };
        // The wire width, not `align_of::<T>()`
        self.align_to(
            fields::scalar_size(T::TYPE_ID).expect("a scalar field has a scalar type_id"),
        );
        let def = FieldDefBuilder::new(f.id(), T::TYPE_ID)
            .offset(self.cursor)
            .size(f.size())
            .flags(f.flags())
            .def_index(def_index)
            .scale(if T::TYPE_ID == Decimal::TYPE_ID {
                -9
            } else {
                0
            })
            .build();
        self.push(def, name);
    }

    fn string<T: FieldType>(&mut self, f: Field<T>, name: &str, width: u16) {
        // strings are 1-byte aligned, so no padding
        let def = FieldDefBuilder::new(f.id(), T::TYPE_ID)
            .offset(self.cursor)
            .size(width)
            .flags(f.flags())
            .build();
        self.push(def, name);
    }

    fn nested(&mut self, field_id: u16, name: &str, nested: Nested) {
        self.align_to(nested.align);
        let def = FieldDefBuilder::new(field_id, fields::STRUCT_ID)
            .offset(self.cursor)
            .size(nested.size)
            .flags(nested.flags)
            .def_index(nested.struct_idx)
            .build();
        self.push(def, name);
    }

    fn at(&mut self, offset: usize) {
        let offset = u16::try_from(offset).expect("record layout exceeds u16 bytes");
        assert!(
            offset >= self.cursor,
            "offset {offset} overlaps the field ending at {}",
            self.cursor
        );
        self.pad_to(offset);
    }

    fn align_to(&mut self, align: u16) {
        self.pad_to(align_up(self.cursor, align));
    }

    fn pad_to(&mut self, offset: u16) {
        if offset > self.cursor {
            self.push(
                FieldDefBuilder::padding(self.cursor, offset - self.cursor),
                "",
            );
        }
    }

    fn seal_base(&mut self) -> u16 {
        if let Some(base) = self.base {
            return base;
        }
        self.align_to(8);
        self.base = Some(self.cursor);
        self.cursor
    }

    fn finish_record(&mut self) -> u16 {
        let base = self.seal_base();
        self.align_to(8);
        base
    }

    fn push(&mut self, mut def: FieldDef, name: &str) {
        if self.base.is_some() && !def.is_padding() {
            def.flags |= fields::FLAG_IS_EXTENSION;
        }
        self.cursor = self
            .cursor
            .checked_add(def.size)
            .expect("record layout exceeds u16 bytes");
        self.field_defs.push(def);
        self.field_names.push(name.to_owned());
    }

    /// Round the record or struct up to an 8-byte boundary and return its size.
    fn finish_base(&mut self) -> u16 {
        self.align_to(8);
        self.cursor
    }
}

const fn align_up(v: u16, align: u16) -> u16 {
    (v + align - 1) & !(align - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v4::{
        rtype,
        types::{CStr, Char},
    };

    fn bid_ask(sb: &mut StreamLayoutBuilder) -> u16 {
        let mut s = sb.struct_layout("BidAskPair");
        s.field(Field::<i64>::new(20), "bid_px")
            .field(Field::<i64>::new(21), "ask_px")
            .field(Field::<u32>::new(22), "bid_sz")
            .field(Field::<u32>::new(23), "ask_sz");
        s.finish()
    }

    /// A one-record stream.
    fn sample(fixed_price: bool, sym_name: &str) -> StreamLayout {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        let ba = bid_ask(&mut sb);
        {
            let mut r = sb.record(rtype::MBP_1, 0, "Sample");
            r.field(fields::ORDER_ID, "order_id");
            if fixed_price {
                r.field(fields::PRICE, "price");
            } else {
                r.field(Field::<i64>::new(fields::PRICE.id()), "price");
            }
            r.field(fields::SIZE, "size");
            r.field(fields::FLAGS, "flags");
            r.str_field(Field::<CStr>::new(30), sym_name, 8);
            r.struct_field(40, "levels", ba);
            r.finish();
        }
        sb.build().unwrap()
    }

    #[test]
    fn fixed_price_carries_scale_and_wire_size() {
        let rec = &sample(true, "symbol").record_layouts[0];
        let price = rec
            .field_defs
            .iter()
            .find(|f| f.field_id == fields::PRICE.id())
            .unwrap();
        assert_eq!(price.scale, -9);
        // Fixed is 8 wire bytes, not size_of::<Decimal>()
        assert_eq!(price.size, 8);
    }

    #[test]
    fn padding_and_length_are_unnamed() {
        let layout = sample(true, "symbol");
        let rec = &layout.record_layouts[0];
        for (f, name) in rec.field_defs.iter().zip(&rec.field_names) {
            if f.is_padding() {
                assert!(name.is_empty(), "padding should be unnamed");
            }
        }
        let header = &layout.struct_layouts[0];
        assert_eq!(header.name, "RecordHeader");
        assert_eq!(header.size as usize, std::mem::size_of::<RecordHeader>());
        let name_of = |id: u16| {
            let i = header
                .field_defs
                .iter()
                .position(|f| f.field_id == id)
                .unwrap();
            header.field_names[i].as_str()
        };
        assert_eq!(name_of(fields::LENGTH.id()), ""); // hidden
        assert_eq!(name_of(fields::RTYPE.id()), "rtype");
    }

    #[test]
    fn layout_hash_covers_structure_not_display() {
        let h = |l: &StreamLayout| l.record_layouts[0].layout_hash;
        let base = sample(true, "symbol");
        // Identical builds hash identically
        assert_eq!(h(&base), h(&sample(true, "symbol")));
        // A display-only change (the field name) doesn't change the hash
        assert_eq!(h(&base), h(&sample(true, "other_symbol")));
        // A structural change (price type/scale) does
        assert_ne!(h(&base), h(&sample(false, "symbol")));
    }

    #[test]
    fn scalar_array_field_carries_size_and_array_flag() {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "WithConditions");
            r.field(Field::<[Char; 4]>::new(0x1A), "conditions");
            r.finish();
        }
        let layout = sb.build().unwrap();
        let cond = layout.record_layouts[0]
            .field_defs
            .iter()
            .find(|f| f.field_id == 0x1A)
            .unwrap();
        assert!(cond.is_array());
        assert_eq!(cond.type_id, Char::TYPE_ID);
        assert_eq!(cond.size, 4); // 4 elements x 1 byte
    }

    #[test]
    fn at_pads_a_gap() {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_0, 0, "Gapped");
            r.at(28).field(fields::SIZE, "size");
            r.finish();
        }
        let rec = &sb.build().unwrap().record_layouts[0];
        let gap = rec.field_defs[1];
        assert!(gap.is_padding());
        assert_eq!((gap.offset, gap.size), (24, 4));
        assert_eq!(rec.field_defs[2].offset, 28);
    }

    #[test]
    #[should_panic(expected = "overlaps")]
    fn at_rejects_an_overlap() {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        let mut r = sb.record(rtype::MBP_0, 0, "Overlapping");
        r.field(fields::ORDER_ID, "order_id").at(28);
    }

    #[test]
    fn struct_array_of_spans_all_elements() {
        let mut sb = StreamLayoutBuilder::new(1, 8);
        {
            let mut r = sb.record(rtype::MBP_10, 0, "Mbp10");
            r.field(fields::ORDER_ID, "order_id");
            r.struct_array_of(fields::LEVELS_10, "levels");
            r.finish();
        }
        let layout = sb.build().unwrap();
        let levels = layout.record_layouts[0]
            .field_defs
            .iter()
            .find(|f| f.field_id == fields::LEVELS_10.id())
            .unwrap();
        let ba = &layout.struct_layouts[levels.def_index as usize];
        assert_eq!(ba.name, "BidAskPair");
        assert!(levels.is_array());
        assert_eq!(levels.type_id, fields::STRUCT_ID);
        assert_eq!(levels.size, ba.size * 10);
        let align = ba.align();
        assert_eq!(levels.offset % align, 0);
    }
}
