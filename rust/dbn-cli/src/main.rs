use std::{fs::File, io};

use clap::Parser;
use dbn::{
    decode::{DbnRecordDecoder, DecodeDbn, DynDecoder},
    encode::{json, DynEncoder, EncodeDbn},
    enums::SType,
    MetadataBuilder,
};
use dbn_cli::{infer_encoding_and_compression, output_from_args, Args};

fn write_dbn<R: io::BufRead>(decoder: DynDecoder<R>, args: &Args) -> anyhow::Result<()> {
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

fn write_dbn_frag<R: io::Read>(
    mut decoder: DbnRecordDecoder<R>,
    args: &Args,
) -> anyhow::Result<()> {
    let writer = output_from_args(args)?;
    let (encoding, compression) = infer_encoding_and_compression(args)?;
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
    while let Some(record) = decoder.decode_ref()? {
        // Assume no ts_out for safety
        match unsafe { encoder.encode_record_ref(record, false) } {
            // Handle broken pipe as a non-error.
            Err(dbn::Error::Io { source, .. })
                if source.kind() == std::io::ErrorKind::BrokenPipe =>
            {
                return Ok(());
            }
            res => {
                res?;
            }
        };
        n += 1;
        if args.limit.map_or(false, |l| n >= l.get()) {
            break;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.is_fragment {
        if args.input.as_os_str() == "-" {
            write_dbn_frag(DbnRecordDecoder::new(io::stdin().lock()), &args)
        } else {
            write_dbn_frag(
                DbnRecordDecoder::new(File::open(args.input.clone())?),
                &args,
            )
        }
    } else if args.is_zstd_fragment {
        if args.input.as_os_str() == "-" {
            write_dbn_frag(
                DbnRecordDecoder::new(zstd::stream::Decoder::with_buffer(io::stdin().lock())?),
                &args,
            )
        } else {
            write_dbn_frag(
                DbnRecordDecoder::new(zstd::stream::Decoder::new(File::open(args.input.clone())?)?),
                &args,
            )
        }
    } else if args.input.as_os_str() == "-" {
        write_dbn(DynDecoder::inferred_with_buffer(io::stdin().lock())?, &args)
    } else {
        write_dbn(DynDecoder::from_file(&args.input)?, &args)
    }
}
