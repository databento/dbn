use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use anyhow::{anyhow, Context};
use clap::Parser;
use dbn::decode::{
    DbnMetadata, DbnRecordDecoder, DecodeRecordRef, DynDecoder, MergeDecoder, MergeRecordDecoder,
};
use dbn_cli::{
    encode::{
        encode_from_dbn, encode_from_frag, silence_broken_pipe, split_encode_from_dbn,
        split_encode_from_frag,
    },
    filter::{LimitFilter, SchemaFilter},
    Args,
};

const STDIN_SENTINEL: &str = "-";

fn open_input_file(path: &Path) -> anyhow::Result<File> {
    File::open(path).with_context(|| format!("opening file to decode at path '{}'", path.display()))
}

fn wrap_frag(args: &Args, decoder: impl DecodeRecordRef) -> impl DecodeRecordRef {
    LimitFilter::new_no_metadata(
        SchemaFilter::new_no_metadata(decoder, args.schema_filter),
        args.limit,
    )
}

/// assume no ts_out for fragments
const FRAG_TS_OUT: bool = false;

fn decode_frag(args: &Args, reader: impl io::Read) -> anyhow::Result<impl DecodeRecordRef> {
    Ok(wrap_frag(
        args,
        DbnRecordDecoder::with_version(
            reader,
            args.input_version(),
            args.upgrade_policy(),
            FRAG_TS_OUT,
        )?,
    ))
}

fn wrap(
    args: &Args,
    decoder: impl DecodeRecordRef + DbnMetadata,
) -> impl DecodeRecordRef + DbnMetadata {
    LimitFilter::new(SchemaFilter::new(decoder, args.schema_filter), args.limit)
}

fn with_inputs(args: Args) -> anyhow::Result<()> {
    if args.is_input_fragment {
        let decoders = args
            .input
            .iter()
            .map(|input| {
                Ok(DbnRecordDecoder::with_version(
                    BufReader::new(open_input_file(input)?),
                    args.input_version(),
                    args.upgrade_policy(),
                    FRAG_TS_OUT,
                )?)
            })
            .collect::<anyhow::Result<Vec<DbnRecordDecoder<BufReader<File>>>>>()?;
        encode_from_frag(&args, MergeRecordDecoder::new(decoders)?)
    } else if args.is_input_zstd_fragment {
        let decoders = args
            .input
            .iter()
            .map(|input| {
                Ok(DbnRecordDecoder::with_version(
                    zstd::stream::Decoder::new(open_input_file(input)?)?,
                    args.input_version(),
                    args.upgrade_policy(),
                    FRAG_TS_OUT,
                )?)
            })
            .collect::<anyhow::Result<Vec<DbnRecordDecoder<zstd::stream::Decoder<BufReader<File>>>>>>()?;
        encode_from_frag(&args, MergeRecordDecoder::new(decoders)?)
    } else {
        let decoders = args
            .input
            .iter()
            .map(|input| DynDecoder::from_file(input, args.upgrade_policy()))
            .collect::<dbn::Result<Vec<DynDecoder<BufReader<File>>>>>()?;
        encode_from_dbn(&args, wrap(&args, MergeDecoder::new(decoders)?))
    }
}

fn with_input(args: Args, reader: impl BufRead) -> anyhow::Result<()> {
    if let Some(split_by) = args.split_by {
        let Some(output_pattern) = &args.output_pattern else {
            return Err(anyhow!(
                "Must specify an output pattern when splitting files"
            ));
        };
        if args.is_input_fragment {
            split_encode_from_frag(&args, split_by, output_pattern, decode_frag(&args, reader)?)
        } else if args.is_input_zstd_fragment {
            split_encode_from_frag(
                &args,
                split_by,
                output_pattern,
                decode_frag(&args, zstd::stream::Decoder::with_buffer(reader)?)?,
            )
        } else {
            let decoder = DynDecoder::inferred_with_buffer(reader, args.upgrade_policy())?;
            split_encode_from_dbn(&args, split_by, output_pattern, wrap(&args, decoder))
        }
    } else if args.is_input_fragment {
        encode_from_frag(&args, decode_frag(&args, reader)?)
    } else if args.is_input_zstd_fragment {
        encode_from_frag(
            &args,
            decode_frag(&args, zstd::stream::Decoder::with_buffer(reader)?)?,
        )
    } else {
        encode_from_dbn(
            &args,
            wrap(
                &args,
                DynDecoder::inferred_with_buffer(reader, args.upgrade_policy())?,
            ),
        )
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.input.len() > 1 {
        if args.split_by.is_some() {
            return Err(anyhow!("Can't split by files while merging files"));
        }
        with_inputs(args)
    } else if args.input[0].as_os_str() == STDIN_SENTINEL {
        with_input(args, io::stdin().lock())
    } else {
        let reader = BufReader::new(open_input_file(&args.input[0])?);
        with_input(args, reader)
    }
    .or_else(silence_broken_pipe)
}
