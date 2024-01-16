use std::num::NonZeroU64;

use dbn::{
    decode::{DbnMetadata, DecodeRecordRef},
    RType, Record, RecordRef, Schema,
};

#[derive(Debug)]
pub struct SchemaFilter<D> {
    decoder: D,
    rtype: Option<RType>,
}

impl<D> SchemaFilter<D>
where
    D: DbnMetadata,
{
    pub fn new(mut decoder: D, schema: Option<Schema>) -> Self {
        if let Some(schema) = schema {
            decoder.metadata_mut().schema = Some(schema);
        }
        Self::new_no_metadata(decoder, schema)
    }
}

impl<D> SchemaFilter<D> {
    pub fn new_no_metadata(decoder: D, schema: Option<Schema>) -> Self {
        Self {
            decoder,
            rtype: schema.map(RType::from),
        }
    }
}

impl<D: DbnMetadata> DbnMetadata for SchemaFilter<D> {
    fn metadata(&self) -> &dbn::Metadata {
        self.decoder.metadata()
    }

    fn metadata_mut(&mut self) -> &mut dbn::Metadata {
        self.decoder.metadata_mut()
    }
}

impl<D: DecodeRecordRef> DecodeRecordRef for SchemaFilter<D> {
    fn decode_record_ref(&mut self) -> dbn::Result<Option<dbn::RecordRef>> {
        while let Some(record) = self.decoder.decode_record_ref()? {
            if self
                .rtype
                .map(|rtype| rtype as u8 == record.header().rtype)
                .unwrap_or(true)
            {
                // Safe: casting reference to pointer so the pointer will always be valid.
                // Getting around borrow checker limitation.
                return Ok(Some(unsafe {
                    RecordRef::unchecked_from_header(record.header())
                }));
            }
        }
        Ok(None)
    }
}

#[derive(Debug)]
pub struct LimitFilter<D> {
    decoder: D,
    limit: Option<NonZeroU64>,
    record_count: u64,
}

impl<D> LimitFilter<D>
where
    D: DbnMetadata,
{
    pub fn new(mut decoder: D, limit: Option<NonZeroU64>) -> Self {
        if let Some(limit) = limit {
            let metadata_limit = &mut decoder.metadata_mut().limit;
            if let Some(metadata_limit) = metadata_limit {
                *metadata_limit = (*metadata_limit).min(limit);
            } else {
                *metadata_limit = Some(limit);
            }
        }
        Self::new_no_metadata(decoder, limit)
    }
}

impl<D> LimitFilter<D> {
    pub fn new_no_metadata(decoder: D, limit: Option<NonZeroU64>) -> Self {
        Self {
            decoder,
            limit,
            record_count: 0,
        }
    }
}

impl<D: DbnMetadata> DbnMetadata for LimitFilter<D> {
    fn metadata(&self) -> &dbn::Metadata {
        self.decoder.metadata()
    }

    fn metadata_mut(&mut self) -> &mut dbn::Metadata {
        self.decoder.metadata_mut()
    }
}

impl<D: DecodeRecordRef> DecodeRecordRef for LimitFilter<D> {
    fn decode_record_ref(&mut self) -> dbn::Result<Option<RecordRef>> {
        if self
            .limit
            .map(|limit| self.record_count >= limit.get())
            .unwrap_or(false)
        {
            return Ok(None);
        }
        Ok(self.decoder.decode_record_ref()?.map(|rec| {
            self.record_count += 1;
            rec
        }))
    }
}
