use std::io;

use dbn::{
    decode::{DbnMetadata, DecodeRecordRef},
    encode::{
        json, DbnEncodable, DbnRecordEncoder, DynEncoder, DynWriter, EncodeDbn, EncodeRecordRef,
        EncodeRecordTextExt,
    },
    rtype_dispatch, Compression, Encoding, MetadataBuilder, SType, SymbolIndex,
};

use crate::{infer_encoding, output_from_args, Args};

pub fn silence_broken_pipe(err: anyhow::Error) -> anyhow::Result<()> {
    // Handle broken pipe as a non-error.
    if let Some(err) = err.downcast_ref::<dbn::Error>() {
        if matches!(err, dbn::Error::Io { source, .. } if source.kind() == std::io::ErrorKind::BrokenPipe)
        {
            return Ok(());
        }
    }
    Err(err)
}

pub fn encode_from_dbn<D>(args: &Args, mut decoder: D) -> anyhow::Result<()>
where
    D: DecodeRecordRef + DbnMetadata,
{
    let writer = output_from_args(args)?;
    let (encoding, compression, delimiter) = infer_encoding(args)?;
    if args.should_output_metadata {
        if encoding != Encoding::Json {
            return Err(anyhow::format_err!(
                "Metadata flag is only valid with JSON encoding"
            ));
        }
        json::Encoder::new(
            writer,
            args.should_pretty_print,
            args.should_pretty_print,
            args.should_pretty_print,
        )
        .encode_metadata(decoder.metadata())?;
    } else if args.fragment {
        encode_fragment(decoder, writer, compression)?;
    } else {
        let mut encoder = DynEncoder::builder(writer, encoding, compression, decoder.metadata())
            .delimiter(delimiter)
            .write_header(args.write_header)
            .all_pretty(args.should_pretty_print)
            .with_symbol(args.map_symbols)
            .build()?;
        if args.map_symbols {
            let symbol_map = decoder.metadata().symbol_map()?;
            let ts_out = decoder.metadata().ts_out;
            while let Some(rec) = decoder.decode_record_ref()? {
                let sym = symbol_map.get_for_rec(&rec).map(String::as_str);
                // SAFETY: `ts_out` is accurate because it's sourced from the metadata
                unsafe {
                    encoder.encode_ref_ts_out_with_sym(rec, ts_out, sym)?;
                }
            }
        } else {
            encoder.encode_decoded(decoder)?;
        }
    }
    Ok(())
}

pub fn encode_from_frag<D>(args: &Args, mut decoder: D) -> anyhow::Result<()>
where
    D: DecodeRecordRef,
{
    let writer = output_from_args(args)?;
    let (encoding, compression, delimiter) = infer_encoding(args)?;
    if args.fragment {
        encode_fragment(decoder, writer, compression)?;
        return Ok(());
    }
    assert!(!args.should_output_metadata);

    let mut encoder = DynEncoder::builder(
        writer,
        encoding,
        compression,
        // dummy metadata won't be encoded
        &MetadataBuilder::new()
            .dataset(String::new())
            .schema(None)
            .start(0)
            .stype_in(None)
            .stype_out(SType::InstrumentId)
            .build(),
    )
    .delimiter(delimiter)
    // Can't write header until we know the record type
    .write_header(false)
    .all_pretty(args.should_pretty_print)
    .build()?;
    let mut has_written_header = (encoding != Encoding::Csv) || !args.write_header;
    fn write_header<T: DbnEncodable>(
        _record: &T,
        encoder: &mut DynEncoder<Box<dyn io::Write>>,
    ) -> dbn::Result<()> {
        encoder.encode_header::<T>(false)
    }
    while let Some(record) = decoder.decode_record_ref()? {
        if !has_written_header {
            rtype_dispatch!(record, write_header, &mut encoder)??;
            has_written_header = true;
        }
        encoder.encode_record_ref(record)?;
    }
    Ok(())
}

fn encode_fragment<D: DecodeRecordRef>(
    mut decoder: D,
    writer: Box<dyn io::Write>,
    compression: Compression,
) -> dbn::Result<()> {
    let mut encoder = DbnRecordEncoder::new(DynWriter::new(writer, compression)?);
    while let Some(record) = decoder.decode_record_ref()? {
        encoder.encode_record_ref(record)?;
    }
    Ok(())
}
