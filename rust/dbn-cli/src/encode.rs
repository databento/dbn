use std::io;

use dbn::{
    decode::{DbnRecordDecoder, DecodeDbn, DecodeRecordRef, DynDecoder},
    encode::{
        json, DbnEncodable, DbnRecordEncoder, DynEncoder, DynWriter, EncodeDbn, EncodeRecordRef,
    },
    rtype_dispatch, Compression, Encoding, MetadataBuilder, SType,
};

use crate::{infer_encoding_and_compression, output_from_args, Args};

pub fn encode_from_dbn<R: io::BufRead>(decoder: DynDecoder<R>, args: &Args) -> anyhow::Result<()> {
    let writer = output_from_args(args)?;
    let (encoding, compression) = infer_encoding_and_compression(args)?;
    let encode_res = if args.should_output_metadata {
        assert!(args.json);
        json::Encoder::new(
            writer,
            args.should_pretty_print,
            args.should_pretty_print,
            args.should_pretty_print,
        )
        .encode_metadata(decoder.metadata())
    } else if args.fragment {
        encode_fragment(decoder, writer, compression, args)
    } else if let Some(limit) = args.limit {
        let mut metadata = decoder.metadata().clone();
        // Update metadata
        metadata.limit = args.limit;
        DynEncoder::new(
            writer,
            encoding,
            compression,
            &metadata,
            args.should_pretty_print,
            args.should_pretty_print,
            args.should_pretty_print,
        )?
        .encode_decoded_with_limit(decoder, limit)
    } else {
        DynEncoder::new(
            writer,
            encoding,
            compression,
            decoder.metadata(),
            args.should_pretty_print,
            args.should_pretty_print,
            args.should_pretty_print,
        )?
        .encode_decoded(decoder)
    };
    match encode_res {
        // Handle broken pipe as a non-error.
        Err(dbn::Error::Io { source, .. }) if source.kind() == std::io::ErrorKind::BrokenPipe => {
            Ok(())
        }
        res => Ok(res?),
    }
}

pub fn encode_from_frag<R: io::Read>(
    mut decoder: DbnRecordDecoder<R>,
    args: &Args,
) -> anyhow::Result<()> {
    let writer = output_from_args(args)?;
    let (encoding, compression) = infer_encoding_and_compression(args)?;
    if args.fragment {
        encode_fragment(decoder, writer, compression, args)?;
        return Ok(());
    }
    assert!(!args.should_output_metadata);

    let mut encoder = DynEncoder::new(
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
        args.should_pretty_print,
        args.should_pretty_print,
        args.should_pretty_print,
    )?;
    let mut n = 0;
    let mut has_written_header = encoding != Encoding::Csv;
    fn write_header<T: DbnEncodable>(
        _record: &T,
        encoder: &mut DynEncoder<Box<dyn io::Write>>,
    ) -> dbn::Result<()> {
        encoder.encode_header::<T>(false)
    }
    while let Some(record) = decoder.decode_record_ref()? {
        if !has_written_header {
            match rtype_dispatch!(record, write_header, &mut encoder)? {
                Err(dbn::Error::Io { source, .. })
                    if source.kind() == io::ErrorKind::BrokenPipe =>
                {
                    return Ok(())
                }
                res => res?,
            }
            has_written_header = true;
        }
        // Assume no ts_out for safety
        match encoder.encode_record_ref(record) {
            // Handle broken pipe as a non-error.
            Err(dbn::Error::Io { source, .. }) if source.kind() == io::ErrorKind::BrokenPipe => {
                return Ok(());
            }
            res => res?,
        };
        n += 1;
        if args.limit.map_or(false, |l| n >= l.get()) {
            break;
        }
    }
    Ok(())
}

fn encode_fragment<D: DecodeRecordRef>(
    mut decoder: D,
    writer: Box<dyn io::Write>,
    compression: Compression,
    args: &Args,
) -> dbn::Result<()> {
    // FIXME: refactor boxed writer and dyn writer
    let mut encoder = DbnRecordEncoder::new(DynWriter::new(writer, compression)?);
    let mut n = 0;
    while let Some(record) = decoder.decode_record_ref()? {
        encoder.encode_record_ref(record)?;
        n += 1;
        if args.limit.map_or(false, |l| n >= l.get()) {
            break;
        }
    }
    Ok(())
}
