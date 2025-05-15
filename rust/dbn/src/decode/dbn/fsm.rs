//! State machine for sans-I/O decoding of DBN for use in higher-level
//! sync and async decoders.

use std::{
    mem::{self, transmute},
    num::NonZeroU64,
    str::Utf8Error,
};

use oval::Buffer;

use crate::{
    decode::{
        dbn::{decode_iso8601, DBN_PREFIX, DBN_PREFIX_LEN},
        FromLittleEndianSlice,
    },
    record::transmute_record_bytes,
    rtype, v1, v2, Error, HasRType, MappingInterval, Metadata, Record, RecordHeader, RecordRef,
    Result, SType, Schema, SymbolMapping, VersionUpgradePolicy, WithTsOut, DBN_VERSION,
    MAX_RECORD_LEN, METADATA_FIXED_LEN, NULL_SCHEMA, NULL_STYPE, UNDEF_TIMESTAMP,
};

/// State machine for decoding DBN with bring your own I/O.
#[derive(Debug)]
pub struct DbnFsm {
    input_dbn_version: u8,
    upgrade_policy: VersionUpgradePolicy,
    ts_out: bool,
    state: State,
    buffer: oval::Buffer,
    compat_buffer: oval::Buffer,
}

impl DbnFsm {
    /// The default internal buffer size: 64 KiB.
    pub const DEFAULT_BUF_SIZE: usize = 64 * (1 << 10);
    const METADATA_PRELUDE_LEN: usize = 8;
    const HEADER_LEN: usize = mem::size_of::<RecordHeader>();
    const U32_SIZE: usize = mem::size_of::<u32>();

    /// Creates a new decoder with the specified buffer sizes.
    pub fn new(buffer_size: usize, compat_size: usize) -> Self {
        Self {
            input_dbn_version: DBN_VERSION,
            ts_out: false,
            upgrade_policy: VersionUpgradePolicy::default(),
            state: State::default(),
            buffer: Buffer::with_capacity(buffer_size),
            compat_buffer: Buffer::with_capacity(compat_size),
        }
    }

    /// Returns the input DBN version.
    pub fn input_dbn_version(&self) -> u8 {
        self.input_dbn_version
    }

    /// Sets the DBN version to expect when decoding.
    ///
    /// # Errors
    /// This function will return an error if the `version` exceeds the highest
    /// supported version.
    pub fn set_input_dbn_version(&mut self, version: u8) -> Result<&mut Self> {
        if version == 0 || version > DBN_VERSION {
            return Err(Error::BadArgument {
                param_name: "version".to_owned(),
                desc: format!("invalid, must be between 1 and {DBN_VERSION}, inclusive"),
            });
        }
        self.input_dbn_version = version;
        Ok(self)
    }

    /// Returns `true` if input has the send timestamp `ts_out` appended to each record.
    pub fn ts_out(&self) -> bool {
        self.ts_out
    }

    /// Sets whether each record is expected to have `ts_out` appended.
    pub fn set_ts_out(&mut self, ts_out: bool) -> &mut Self {
        self.ts_out = ts_out;
        self
    }

    /// Returns the current DBN version upgrade policy.
    pub fn upgrade_policy(&self) -> VersionUpgradePolicy {
        self.upgrade_policy
    }

    /// Sets the DBN version upgrade policy.
    pub fn set_upgrade_policy(&mut self, upgrade_policy: VersionUpgradePolicy) -> &mut Self {
        self.upgrade_policy = upgrade_policy;
        self
    }

    /// Skips the decoder state ahead to record decoding.
    ///
    /// # Preconditions
    /// This method should only be called before any processing.
    pub fn skip_metadata(&mut self) -> &mut Self {
        self.state = State::Record;
        self
    }

    /// Returns the most recently returned record if exists.
    pub fn last_record(&self) -> Option<RecordRef> {
        match self.state {
            State::Prelude | State::Metadata { .. } | State::Record => None,
            // `process_records` is incompatible with this method as there
            // are multiple records to be read
            State::Consume { compat_fill, .. } if compat_fill > 0 => None,
            State::Consume { compat, .. } if compat > 0 => {
                // SAFETY: previously validated as record
                Some(unsafe { RecordRef::new(self.compat_buffer.data()) })
            }
            // SAFETY: previously validated as record
            State::Consume { .. } => Some(unsafe { RecordRef::new(self.buffer.data()) }),
        }
    }

    /// Returns the unprocessed data in the buffer.
    pub fn data(&self) -> &[u8] {
        self.buffer.data()
    }

    /// Returns the mutable slice to all writable space in the buffer.
    pub fn space(&mut self) -> &mut [u8] {
        self.buffer.space()
    }

    /// Should be called after writing to [`space()`](Self::space) to
    /// indicate how many bytes were written.
    pub fn fill(&mut self, nbytes: usize) {
        self.buffer.fill(nbytes);
    }

    /// Copies the given `bytes` to the internal buffer.
    pub fn write_all(&mut self, bytes: &[u8]) {
        self.buffer.grow(self.buffer.capacity() + bytes.len());
        self.space()[..bytes.len()].copy_from_slice(bytes);
        self.fill(bytes.len());
    }

    /// Ensure the compatibility buffer has at least `size` bytes.
    pub fn grow_compat(&mut self, size: usize) {
        self.compat_buffer.grow(size);
    }

    /// Skips ahead `nbytes`. Returns the actual number of bytes skipped.
    pub fn skip(&mut self, nbytes: usize) -> usize {
        match self.state {
            State::Consume {
                read,
                compat,
                compat_fill,
                expand_compat,
            } => {
                self.consume(read, compat, compat_fill, expand_compat);
                self.buffer.consume(nbytes)
            }
            _ => self.buffer.consume(nbytes),
        }
    }

    /// Process some data if available. This method should be called repeatedly until
    /// [`ProcessResult::ReadMore`] is returned.
    ///
    /// # Errors
    /// This function returns an error if it encounters invalid metadata or an invalid
    /// record.
    pub fn process(&mut self) -> ProcessResult<()> {
        loop {
            let available_data = self.buffer.available_data();
            let data = self.buffer.data();
            match self.state {
                State::Prelude if available_data < Self::METADATA_PRELUDE_LEN => {
                    return ProcessResult::ReadMore(Self::METADATA_PRELUDE_LEN - available_data);
                }
                State::Prelude => {
                    if let Err(err) = self.decode_prelude() {
                        return ProcessResult::Err(err);
                    }
                }
                State::Metadata { length } if available_data < length as usize => {
                    return ProcessResult::ReadMore(length as usize - available_data);
                }
                State::Metadata { length } => {
                    return match self.decode_metadata(length) {
                        Ok(metadata) => ProcessResult::Metadata(metadata),
                        Err(err) => ProcessResult::Err(err),
                    };
                }
                State::Record if available_data < Self::HEADER_LEN => {
                    return ProcessResult::ReadMore(Self::HEADER_LEN - available_data)
                }
                State::Record => {
                    let length = data[0] as usize * RecordHeader::LENGTH_MULTIPLIER;
                    if length < Self::HEADER_LEN {
                        return ProcessResult::Err(Error::decode(format!(
                            "invalid record with impossible length {length} which is shorter than the header"
                        )));
                    }
                    if length > available_data {
                        return ProcessResult::ReadMore(length - available_data);
                    }
                    let prev_compat_cap = self.compat_buffer.available_space();
                    let (rem_compat_buffer, rec) = unsafe {
                        Self::upgrade_record(
                            self.input_dbn_version,
                            self.upgrade_policy,
                            self.ts_out,
                            self.buffer.data(),
                            self.compat_buffer.space(),
                        )
                    };
                    if rec.is_none() {
                        self.double_compat_buffer();
                        continue;
                    };
                    let compat_bytes = prev_compat_cap - rem_compat_buffer.len();
                    self.compat_buffer.fill(compat_bytes);
                    self.state = State::Consume {
                        read: length as u32,
                        compat: compat_bytes as u32,
                        compat_fill: 0,
                        expand_compat: false,
                    };
                    return ProcessResult::Record(());
                }
                State::Consume {
                    read,
                    compat,
                    compat_fill,
                    expand_compat,
                } => self.consume(read, compat, compat_fill, expand_compat),
            }
        }
    }

    fn double_compat_buffer(&mut self) {
        self.compat_buffer
            .grow(MAX_RECORD_LEN.max(self.compat_buffer.capacity() * 2));
    }

    /// Reads all available records into the given `rec_refs` vec or until the optional
    /// `limit` is reached, returning the number of records read.
    ///
    /// This method can be  used for batch processing of records  that's not possible
    ///with repeated calls to `process` due to mutable lifetimes.
    ///
    /// # Errors
    /// This function returns an error if it encounters invalid metadata or an invalid
    /// record.
    pub fn process_all<'a>(
        &'a mut self,
        rec_refs: &mut Vec<RecordRef<'a>>,
        limit: Option<NonZeroU64>,
    ) -> ProcessResult<u64> {
        self.process_multiple((rec_refs, limit))
    }

    /// Reads available records into the given `rec_refs` slice until the internal buffer is exhausted or the slice is filled. Returns a mutable slice to the records that have been decoded.
    ///
    /// This method can be  used for batch processing of records that's not possible
    /// with repeated calls to `process` due to mutable lifetimes. Unlike `process_all()`
    /// it can populate `rec_refs` on the stack.
    ///
    /// # Errors
    /// This function returns an error if it encounters invalid metadata or an invalid
    /// record.
    pub fn process_many<'a>(
        &'a mut self,
        rec_refs: &'a mut [Option<RecordRef<'a>>],
    ) -> ProcessResult<&'a mut [RecordRef<'a>]> {
        self.process_multiple(rec_refs)
    }

    fn process_multiple<'a, B>(&'a mut self, mut rec_ref_buf: B) -> ProcessResult<B::Return>
    where
        B: RecRefBuf<'a>,
    {
        // Loop to get to `Record` state
        loop {
            let available_data = self.buffer.available_data();
            // Get through non-`Record` states
            match self.state {
                State::Record => break,
                State::Prelude if available_data < Self::METADATA_PRELUDE_LEN => {
                    return ProcessResult::ReadMore(Self::METADATA_PRELUDE_LEN - available_data);
                }
                State::Prelude => {
                    if let Err(err) = self.decode_prelude() {
                        return ProcessResult::Err(err);
                    }
                }
                State::Metadata { length } if available_data < length as usize => {
                    return ProcessResult::ReadMore(length as usize - available_data);
                }
                State::Metadata { length } => {
                    return match self.decode_metadata(length) {
                        Ok(metadata) => ProcessResult::Metadata(metadata),
                        Err(err) => ProcessResult::Err(err),
                    };
                }
                State::Consume {
                    read,
                    compat,
                    compat_fill,
                    expand_compat,
                } => self.consume(read, compat, compat_fill, expand_compat),
            }
        }
        let mut record_count = 0;
        let mut read_bytes = 0;
        let mut compat_bytes = 0;
        let mut remaining_compat = self.compat_buffer.space();
        let mut expand_compat = false;
        while rec_ref_buf.has_capacity(record_count) {
            if read_bytes >= self.buffer.available_data() {
                break;
            }
            let remaining_data = &self.buffer.data()[read_bytes..];

            let length = remaining_data[0] as usize * RecordHeader::LENGTH_MULTIPLIER;
            if length < Self::HEADER_LEN {
                return ProcessResult::Err(Error::decode(format!(
                    "invalid record with length {length} shorter than header"
                )));
            }
            if length > remaining_data.len() {
                break;
            }
            let prev_compat_cap = remaining_compat.len();
            let (new_rem_compat, rec) = unsafe {
                Self::upgrade_record(
                    self.input_dbn_version,
                    self.upgrade_policy,
                    self.ts_out,
                    remaining_data,
                    remaining_compat,
                )
            };
            let Some(rec) = rec else {
                // Insufficient remaining compat space
                expand_compat = true;
                break;
            };
            rec_ref_buf.push(record_count, rec);
            record_count += 1;
            read_bytes += length;
            // Update compat buffer with split borrow
            remaining_compat = new_rem_compat;
            compat_bytes += prev_compat_cap - remaining_compat.len();
        }
        self.state = State::Consume {
            read: read_bytes as u32,
            compat: compat_bytes as u32,
            compat_fill: compat_bytes as u32,
            expand_compat,
        };
        ProcessResult::Record(rec_ref_buf.finalize(record_count))
    }

    fn consume(&mut self, read: u32, compat: u32, compat_fill: u32, expand_compat: bool) {
        self.buffer.consume(read as usize);
        if compat_fill > 0 {
            self.compat_buffer.fill(compat as usize);
        }
        if compat > 0 {
            self.compat_buffer.consume(compat as usize);
        }
        if expand_compat {
            self.double_compat_buffer();
        }
        self.state = State::Record;
    }

    fn decode_prelude(&mut self) -> crate::Result<()> {
        let data = self.buffer.data();
        if &data[..DBN_PREFIX_LEN] != DBN_PREFIX {
            return Err(Error::decode("invalid DBN header"));
        }
        self.input_dbn_version = data[DBN_PREFIX_LEN];
        if self.input_dbn_version > DBN_VERSION {
            return Err(Error::decode(format!("can't decode newer version of DBN. Decoder version is {DBN_VERSION}, input version is {}", self.input_dbn_version)));
        }
        let length = u32::from_le_slice(&data[4..]);
        if (length as usize) < METADATA_FIXED_LEN {
            return Err(Error::decode(
                "invalid DBN metadata. Metadata length shorter than fixed length.",
            ));
        }
        self.state = State::Metadata { length };
        self.buffer.consume_noshift(Self::METADATA_PRELUDE_LEN);
        self.buffer
            .grow(length as usize + Self::METADATA_PRELUDE_LEN);
        Ok(())
    }

    fn decode_metadata(&mut self, length: u32) -> Result<Metadata> {
        let mut metadata = Self::decode_metadata_impl(self.input_dbn_version, self.buffer.data())?;
        metadata.upgrade(self.upgrade_policy);
        self.buffer.consume(length as usize);
        // Need to shift to ensure record alignment
        self.buffer.shift();
        self.state = State::Record;
        Ok(metadata)
    }

    fn decode_metadata_impl(input_version: u8, buffer: &[u8]) -> Result<Metadata> {
        const U64_SIZE: usize = mem::size_of::<u64>();

        let mut pos = 0;
        let dataset = std::str::from_utf8(&buffer[pos..pos + crate::METADATA_DATASET_CSTR_LEN])
            .map_err(|e| crate::Error::utf8(e, "reading dataset from metadata"))?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        pos += crate::METADATA_DATASET_CSTR_LEN;

        let raw_schema = u16::from_le_slice(&buffer[pos..]);
        let schema = if raw_schema == NULL_SCHEMA {
            None
        } else {
            Some(Schema::try_from(raw_schema).map_err(|_| {
                crate::Error::conversion::<Schema>(format!("{:?}", &buffer[pos..pos + 2]))
            })?)
        };
        pos += mem::size_of::<Schema>();
        let start = u64::from_le_slice(&buffer[pos..]);
        pos += U64_SIZE;
        let end = u64::from_le_slice(&buffer[pos..]);
        pos += U64_SIZE;
        let limit = NonZeroU64::new(u64::from_le_slice(&buffer[pos..]));
        pos += U64_SIZE;
        if input_version == 1 {
            // skip deprecated record_count
            pos += U64_SIZE;
        }
        let stype_in = if buffer[pos] == NULL_STYPE {
            None
        } else {
            Some(
                SType::try_from(buffer[pos])
                    .map_err(|_| crate::Error::conversion::<SType>(buffer[pos]))?,
            )
        };
        pos += mem::size_of::<SType>();
        let stype_out = SType::try_from(buffer[pos])
            .map_err(|_| crate::Error::conversion::<SType>(buffer[pos]))?;
        pos += mem::size_of::<SType>();
        let ts_out = buffer[pos] != 0;
        pos += mem::size_of::<bool>();
        let symbol_cstr_len = if input_version == 1 {
            v1::SYMBOL_CSTR_LEN
        } else {
            let res = u16::from_le_slice(&buffer[pos..]);
            pos += mem::size_of::<u16>();
            res as usize
        };
        // skip reserved
        pos += if input_version == 1 {
            v1::METADATA_RESERVED_LEN
        } else {
            crate::METADATA_RESERVED_LEN
        };
        let schema_definition_length = u32::from_le_slice(&buffer[pos..]);
        if schema_definition_length != 0 {
            return Err(crate::Error::decode(
                "this version of dbn can't parse schema definitions",
            ));
        }
        pos += Self::U32_SIZE + (schema_definition_length as usize);
        let symbols =
            Self::decode_metadata_repeated_symbol_cstr(symbol_cstr_len, buffer, &mut pos)?;
        let partial =
            Self::decode_metadata_repeated_symbol_cstr(symbol_cstr_len, buffer, &mut pos)?;
        let not_found =
            Self::decode_metadata_repeated_symbol_cstr(symbol_cstr_len, buffer, &mut pos)?;
        let mappings = Self::decode_metadata_symbol_mappings(symbol_cstr_len, buffer, &mut pos)?;

        Ok(Metadata {
            version: input_version,
            dataset,
            schema,
            stype_in,
            stype_out,
            start,
            end: if end == UNDEF_TIMESTAMP {
                None
            } else {
                NonZeroU64::new(end)
            },
            limit,
            ts_out,
            symbol_cstr_len,
            symbols,
            partial,
            not_found,
            mappings,
        })
    }

    fn decode_metadata_repeated_symbol_cstr(
        symbol_cstr_len: usize,
        buffer: &[u8],
        pos: &mut usize,
    ) -> crate::Result<Vec<String>> {
        if *pos + Self::U32_SIZE > buffer.len() {
            return Err(crate::Error::decode(
                "unexpected end of metadata buffer in symbol cstr",
            ));
        }
        let count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = count * symbol_cstr_len;
        if *pos + read_size > buffer.len() {
            return Err(crate::Error::decode(
                "unexpected end of metadata buffer in symbol cstr",
            ));
        }
        let mut res = Vec::with_capacity(count);
        for i in 0..count {
            res.push(
                Self::decode_metadata_symbol(symbol_cstr_len, buffer, pos)
                    .map_err(|e| crate::Error::utf8(e, format!("decoding symbol at index {i}")))?,
            );
        }
        Ok(res)
    }

    fn decode_metadata_symbol_mappings(
        symbol_cstr_len: usize,
        buffer: &[u8],
        pos: &mut usize,
    ) -> crate::Result<Vec<SymbolMapping>> {
        if *pos + Self::U32_SIZE > buffer.len() {
            return Err(crate::Error::decode(
                "unexpected end of metadata buffer in symbol mapping",
            ));
        }
        let count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let mut res = Vec::with_capacity(count);
        // Because each `SymbolMapping` itself is of a variable length, decoding it requires frequent bounds checks
        for i in 0..count {
            res.push(Self::decode_metadata_symbol_mapping(
                symbol_cstr_len,
                i,
                buffer,
                pos,
            )?);
        }
        Ok(res)
    }

    fn decode_metadata_symbol_mapping(
        symbol_cstr_len: usize,
        idx: usize,
        buffer: &[u8],
        pos: &mut usize,
    ) -> crate::Result<SymbolMapping> {
        let min_symbol_mapping_encoded_len = symbol_cstr_len + mem::size_of::<u32>();
        let mapping_interval_encoded_len = mem::size_of::<u32>() * 2 + symbol_cstr_len;
        if *pos + min_symbol_mapping_encoded_len > buffer.len() {
            return Err(crate::Error::decode(format!(
                "unexpected end of metadata buffer while parsing symbol mapping at index {idx}"
            )));
        }
        let raw_symbol = Self::decode_metadata_symbol(symbol_cstr_len, buffer, pos)
            .map_err(|e| crate::Error::utf8(e, "parsing raw symbol"))?;
        let interval_count = u32::from_le_slice(&buffer[*pos..]) as usize;
        *pos += Self::U32_SIZE;
        let read_size = interval_count * mapping_interval_encoded_len;
        if *pos + read_size > buffer.len() {
            return Err(crate::Error::decode(format!(
                "symbol mapping at index {idx} with `interval_count` {interval_count} doesn't match size of buffer \
                which only contains space for {} intervals",
                (buffer.len() - *pos) / mapping_interval_encoded_len
            )));
        }
        let mut intervals = Vec::with_capacity(interval_count);
        for i in 0..interval_count {
            let raw_start_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += Self::U32_SIZE;
            let start_date = decode_iso8601(raw_start_date).map_err(|e| {
                crate::Error::decode(format!("{e} while parsing start date of mapping interval at index {i} within mapping at index {idx}"))
            })?;
            let raw_end_date = u32::from_le_slice(&buffer[*pos..]);
            *pos += Self::U32_SIZE;
            let end_date = decode_iso8601(raw_end_date).map_err(|e| {
                crate::Error::decode(format!("{e} while parsing end date of mapping interval at index {i} within mapping at index {idx}"))
            })?;
            let symbol = Self::decode_metadata_symbol(symbol_cstr_len, buffer, pos).map_err(|e| {
                crate::Error::utf8(e, format!("parsing symbol for mapping interval at index {i} within mapping at index {idx}"))
            })?;
            intervals.push(MappingInterval {
                start_date,
                end_date,
                symbol,
            });
        }
        Ok(SymbolMapping {
            raw_symbol,
            intervals,
        })
    }

    fn decode_metadata_symbol(
        symbol_cstr_len: usize,
        buffer: &[u8],
        pos: &mut usize,
    ) -> std::result::Result<String, Utf8Error> {
        let symbol_slice = &buffer[*pos..*pos + symbol_cstr_len];
        let symbol = std::str::from_utf8(symbol_slice)?
            // remove null bytes
            .trim_end_matches('\0')
            .to_owned();
        *pos += symbol_cstr_len;
        Ok(symbol)
    }

    unsafe fn upgrade_record<'a, 'b>(
        version: u8,
        upgrade_policy: VersionUpgradePolicy,
        ts_out: bool,
        read_buffer: &'a [u8],
        compat_buffer: &'a mut [u8],
    ) -> (&'b mut [u8], Option<RecordRef<'a>>)
    where
        'a: 'b,
    {
        let rec = RecordRef::new(read_buffer);
        match (version, upgrade_policy) {
            (1, VersionUpgradePolicy::UpgradeToV2) => match rec.header().rtype {
                rtype::INSTRUMENT_DEF => {
                    return upgrade_record::<v1::InstrumentDefMsg, v2::InstrumentDefMsg>(
                        ts_out,
                        compat_buffer,
                        read_buffer,
                    );
                }
                rtype::SYMBOL_MAPPING => {
                    return upgrade_record::<v1::SymbolMappingMsg, v2::SymbolMappingMsg>(
                        ts_out,
                        compat_buffer,
                        read_buffer,
                    );
                }
                rtype::ERROR => {
                    return upgrade_record::<v1::ErrorMsg, v2::ErrorMsg>(
                        ts_out,
                        compat_buffer,
                        read_buffer,
                    );
                }
                rtype::SYSTEM => {
                    return upgrade_record::<v1::SystemMsg, v2::SystemMsg>(
                        ts_out,
                        compat_buffer,
                        read_buffer,
                    );
                }
                _ => (),
            },
            (2, VersionUpgradePolicy::UpgradeToV2) => {}
            (..=DBN_VERSION, VersionUpgradePolicy::AsIs) => {}
            _ => unreachable!("Unsupported version {version}"),
        }
        (compat_buffer, Some(rec))
    }
}

trait RecRefBuf<'a> {
    type Return;

    fn has_capacity(&self, record_count: usize) -> bool;
    fn push(&mut self, record_count: usize, rec_ref: RecordRef<'a>);
    fn finalize(self, record_count: usize) -> Self::Return;
}

impl<'a> RecRefBuf<'a> for (&mut Vec<RecordRef<'a>>, Option<NonZeroU64>) {
    type Return = u64;

    fn has_capacity(&self, record_count: usize) -> bool {
        self.1.is_none_or(|l| l.get() > record_count as u64)
    }

    fn push(&mut self, _: usize, rec_ref: RecordRef<'a>) {
        self.0.push(rec_ref);
    }

    fn finalize(self, record_count: usize) -> Self::Return {
        record_count as u64
    }
}

impl<'a> RecRefBuf<'a> for &'a mut [Option<RecordRef<'a>>] {
    type Return = &'a mut [RecordRef<'a>];

    fn has_capacity(&self, record_count: usize) -> bool {
        record_count < self.len()
    }

    fn push(&mut self, record_count: usize, rec_ref: RecordRef<'a>) {
        self[record_count] = Some(rec_ref);
    }

    fn finalize(self, record_count: usize) -> Self::Return {
        // SAFETY: `record_count` records in `rec_refs` have been populated so it's safe to cast
        // these `Option<RecordRef>` to `RecordRef`
        unsafe {
            transmute::<&mut [Option<RecordRef>], &mut [RecordRef]>(&mut self[..record_count])
        }
    }
}

unsafe fn upgrade_record<'a, T, U>(
    ts_out: bool,
    compat_buffer: &'a mut [u8],
    input: &'a [u8],
) -> (&'a mut [u8], Option<RecordRef<'a>>)
where
    T: HasRType,
    U: AsRef<[u8]> + HasRType + for<'t> From<&'t T>,
{
    if ts_out {
        let rec = transmute_record_bytes::<WithTsOut<T>>(input).unwrap();
        let upgraded = WithTsOut::new(U::from(&rec.rec), rec.ts_out);
        if std::mem::size_of_val(&upgraded) >= compat_buffer.len() {
            return (compat_buffer, None);
        };
        // Split at to have multiple mutable borrows to the same buffer, each
        // with their own unique slice within the buffer
        let (record_compat, rem_compat) =
            compat_buffer.split_at_mut(std::mem::size_of_val(&upgraded));
        record_compat.copy_from_slice(upgraded.as_ref());
        (rem_compat, Some(RecordRef::new(record_compat)))
    } else {
        let upgraded = U::from(transmute_record_bytes::<T>(input).unwrap());
        if std::mem::size_of_val(&upgraded) >= compat_buffer.len() {
            return (compat_buffer, None);
        };
        let (record_compat, rem_compat) =
            compat_buffer.split_at_mut(std::mem::size_of_val(&upgraded));
        record_compat.copy_from_slice(upgraded.as_ref());
        (rem_compat, Some(RecordRef::new(record_compat)))
    }
}

impl Default for DbnFsm {
    fn default() -> Self {
        Self {
            input_dbn_version: DBN_VERSION,
            ts_out: false,
            upgrade_policy: VersionUpgradePolicy::default(),
            state: State::default(),
            buffer: Buffer::with_capacity(Self::DEFAULT_BUF_SIZE),
            compat_buffer: Buffer::with_capacity(Self::DEFAULT_BUF_SIZE),
        }
    }
}

#[derive(Debug, Default)]
enum State {
    #[default]
    Prelude,
    Metadata {
        length: u32,
    },
    Record,
    /// Advance internal buffer state. Gets around mutability requirements.
    Consume {
        /// Bytes read from `buffer`.
        read: u32,
        /// Bytes read from `compat_buffer`.
        compat: u32,
        /// Bytes written to `compat_buffer`. Used for process_records where the
        /// compat_buffer can't be modified at all.
        compat_fill: u32,
        /// `compat_buffer` capacity should be expanded
        expand_compat: bool,
    },
}

/// The decoding result from a call to [`DbnFsm::process()`], [`DbnFsm::process_all()`],
/// and [`DbnFsm::process_many()`].
#[derive(Debug)]
pub enum ProcessResult<R> {
    /// More data should be read into `space()`.
    ReadMore(usize),
    /// Decoded the metadata header.
    Metadata(Metadata),
    /// Decoded a record in the case of [`DbnFsm::process()`], which can be accessed
    /// through [`DbnFsm::last_record()`]. Decoded one or more records in the case of
    /// [`DbnFsm::process_all()`] or [`DbnFsm::process_many()`].
    Record(R),
    /// Failed to decode.
    Err(Error),
}

#[cfg(test)]
mod tests {
    use std::ffi::c_char;

    use crate::{
        encode::{DbnMetadataEncoder, DbnRecordEncoder, EncodeRecord},
        Dataset, Mbp1Msg, SType, Schema, TradeMsg, MAX_RECORD_LEN, SYMBOL_CSTR_LEN,
    };
    use rstest::*;
    use time::{
        macros::{date, datetime},
        OffsetDateTime,
    };

    use super::*;

    #[test]
    fn test_decode_metadata_symbol_invalid_utf8() {
        let mut bytes = [0; SYMBOL_CSTR_LEN];
        // continuation byte
        bytes[0] = 0x80;
        let mut pos = 0;
        let res = DbnFsm::decode_metadata_symbol(bytes.len(), bytes.as_slice(), &mut pos);
        assert!(res.is_err());
    }

    #[test]
    fn test_decode_metadata_symbol() {
        let bytes = b"SPX.1.2\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(bytes.len(), v1::SYMBOL_CSTR_LEN);
        let mut pos = 0;
        let res = DbnFsm::decode_metadata_symbol(v1::SYMBOL_CSTR_LEN, bytes.as_slice(), &mut pos)
            .unwrap();
        assert_eq!(pos, v1::SYMBOL_CSTR_LEN);
        assert_eq!(&res, "SPX.1.2");
    }

    #[rstest]
    fn test_process_all(
        #[values(7, 64, 1_024, 16_384, DbnFsm::DEFAULT_BUF_SIZE, 1 << 20)] chunk_size: usize,
        #[values(VersionUpgradePolicy::AsIs, VersionUpgradePolicy::UpgradeToV2)]
        upgrade_policy: VersionUpgradePolicy,
        #[values(MAX_RECORD_LEN, 4_096, DbnFsm::DEFAULT_BUF_SIZE)] buffer_size: usize,
        #[values(0, MAX_RECORD_LEN, 4_096, DbnFsm::DEFAULT_BUF_SIZE)] compat_size: usize,
        #[values(None, NonZeroU64::new(5), NonZeroU64::new(16))] limit: Option<NonZeroU64>,
        #[values(true, false)] has_metadata: bool,
    ) {
        let mut data = Vec::new();
        let start_date = date!(2025 - 05 - 15);
        let end_date = date!(2025 - 05 - 17);
        let metadata = Metadata::builder()
            .dataset(Dataset::EqusMini)
            .schema(Some(Schema::Trades))
            .stype_in(Some(SType::RawSymbol))
            .stype_out(SType::InstrumentId)
            .start(datetime!(2025-05-15 00:00 UTC).unix_timestamp_nanos() as u64)
            .symbols(vec![
                "AAPL".to_owned(),
                "META".to_owned(),
                "MSFT".to_owned(),
                "NVDA".to_owned(),
            ])
            .mappings(vec![
                SymbolMapping {
                    raw_symbol: "AAPL".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date,
                        end_date,
                        symbol: 1.to_string(),
                    }],
                },
                SymbolMapping {
                    raw_symbol: "META".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date,
                        end_date,
                        symbol: 2.to_string(),
                    }],
                },
                SymbolMapping {
                    raw_symbol: "MSFT".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date,
                        end_date,
                        symbol: 1.to_string(),
                    }],
                },
                SymbolMapping {
                    raw_symbol: "NVDA".to_owned(),
                    intervals: vec![MappingInterval {
                        start_date,
                        end_date,
                        symbol: 1.to_string(),
                    }],
                },
            ])
            .build();
        if has_metadata {
            let mut encoder = DbnMetadataEncoder::new(&mut data);
            encoder.encode(&metadata).unwrap();
        }
        let mut encoder = DbnRecordEncoder::new(&mut data);
        for _ in 0..10_000 {
            encoder.encode_record(&TradeMsg::default()).unwrap();
        }
        let mut target = DbnFsm::new(buffer_size, compat_size);
        target.set_input_dbn_version(1).unwrap();
        if !has_metadata {
            target.skip_metadata();
        }
        target.set_upgrade_policy(upgrade_policy);
        let mut rec_count = 0;
        for slice in data.chunks(chunk_size) {
            target.write_all(slice);
            let mut recs = Vec::new();
            let res = target.process_all(&mut recs, limit);
            dbg!(&res, data.len(), slice.len());
            assert!(!matches!(res, ProcessResult::Err(_)));
            match res {
                ProcessResult::ReadMore(_) => continue,
                ProcessResult::Metadata(decoded_metadata) => {
                    assert!(has_metadata);
                    assert!(recs.is_empty());
                    assert_eq!(decoded_metadata, metadata);
                }
                ProcessResult::Record(processed_count) => {
                    assert!(limit.is_none_or(|l| l.get() >= processed_count));
                    assert_eq!(processed_count, recs.len() as u64);
                    rec_count += recs.len();
                }
                ProcessResult::Err(error) => panic!("unexpected error {error}"),
            }
        }
        loop {
            let mut recs = Vec::new();
            let res = target.process_all(&mut recs, limit);
            dbg!(&res);
            if let ProcessResult::Record(processed_count) = res {
                if processed_count == 0 {
                    break;
                }
                assert!(limit.is_none_or(|l| l.get() >= processed_count));
                assert_eq!(processed_count, recs.len() as u64);
                rec_count += recs.len();
            } else {
                panic!("unexpected result after writing all input");
            }
        }
        assert_eq!(rec_count, 10_000);
    }

    #[test]
    fn test_upgrade_symbol_mapping_ts_out() -> crate::Result<()> {
        let orig = WithTsOut::new(
            v1::SymbolMappingMsg::new(1, 2, "ES.c.0", "ESH4", 0, 0)?,
            OffsetDateTime::now_utc().unix_timestamp_nanos() as u64,
        );
        let mut compat_buffer = [0; MAX_RECORD_LEN];
        let (rem_compat, res) = unsafe {
            DbnFsm::upgrade_record(
                1,
                VersionUpgradePolicy::UpgradeToV2,
                true,
                orig.as_ref(),
                &mut compat_buffer,
            )
        };
        let res = res.unwrap();
        assert_eq!(
            rem_compat.len(),
            MAX_RECORD_LEN - std::mem::size_of::<WithTsOut<v2::SymbolMappingMsg>>()
        );
        assert_eq!(rem_compat.len(), MAX_RECORD_LEN - res.record_size());
        let upgraded = res.get::<WithTsOut<v2::SymbolMappingMsg>>().unwrap();
        assert_eq!(orig.ts_out, upgraded.ts_out);
        assert_eq!(orig.rec.stype_in_symbol()?, upgraded.rec.stype_in_symbol()?);
        assert_eq!(
            orig.rec.stype_out_symbol()?,
            upgraded.rec.stype_out_symbol()?
        );
        assert_eq!(upgraded.record_size(), std::mem::size_of_val(upgraded));
        // used compat buffer
        assert!(std::ptr::addr_eq(upgraded.header(), compat_buffer.as_ptr()));
        Ok(())
    }

    #[test]
    fn test_upgrade_mbp1_ts_out() -> crate::Result<()> {
        let rec = Mbp1Msg {
            price: 1_250_000_000,
            side: b'A' as c_char,
            ..Mbp1Msg::default()
        };
        let orig = WithTsOut::new(rec, OffsetDateTime::now_utc().unix_timestamp_nanos() as u64);
        let mut compat_buffer = [0; MAX_RECORD_LEN];
        let (rem_compat, res) = unsafe {
            DbnFsm::upgrade_record(
                1,
                VersionUpgradePolicy::UpgradeToV2,
                true,
                orig.as_ref(),
                &mut compat_buffer,
            )
        };
        let res = res.unwrap();
        // Unchanged
        assert_eq!(rem_compat.len(), MAX_RECORD_LEN);
        let upgraded = res.get::<WithTsOut<Mbp1Msg>>().unwrap();
        // compat buffer unused and pointer unchanged
        assert!(std::ptr::eq(orig.header(), upgraded.header()));
        Ok(())
    }
}
