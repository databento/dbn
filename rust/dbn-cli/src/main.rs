use std::{
    fs::File,
    io::{self, BufReader},
};

use clap::Parser;
use dbn::decode::{DbnMetadata, DbnRecordDecoder, DecodeRecordRef, DynDecoder};
use dbn_cli::{
    encode::{encode_from_dbn, encode_from_frag},
    filter::{LimitFilter, SchemaFilter},
    Args,
};

const STDIN_SENTINEL: &str = "-";

fn wrap_frag(args: &Args, reader: impl io::Read) -> anyhow::Result<impl DecodeRecordRef> {
    Ok(LimitFilter::new_no_metadata(
        SchemaFilter::new_no_metadata(
            DbnRecordDecoder::with_version(reader, args.input_version(), args.upgrade_policy())?,
            args.schema_filter,
        ),
        args.limit,
    ))
}

fn wrap<R: io::BufRead>(
    args: &Args,
    decoder: DynDecoder<'static, R>,
) -> impl DecodeRecordRef + DbnMetadata {
    LimitFilter::new(SchemaFilter::new(decoder, args.schema_filter), args.limit)
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    // DBN fragment
    if args.is_input_fragment {
        if args.input.as_os_str() == STDIN_SENTINEL {
            encode_from_frag(wrap_frag(&args, io::stdin().lock())?, &args)
        } else {
            encode_from_frag(
                wrap_frag(&args, BufReader::new(File::open(args.input.clone())?))?,
                &args,
            )
        }
    // Zstd-compressed DBN fragment
    } else if args.is_input_zstd_fragment {
        if args.input.as_os_str() == STDIN_SENTINEL {
            encode_from_frag(
                wrap_frag(
                    &args,
                    zstd::stream::Decoder::with_buffer(io::stdin().lock())?,
                )?,
                &args,
            )
        } else {
            encode_from_frag(
                wrap_frag(
                    &args,
                    zstd::stream::Decoder::new(File::open(args.input.clone())?)?,
                )?,
                &args,
            )
        }
    // DBN stream (with metadata)
    } else if args.input.as_os_str() == STDIN_SENTINEL {
        encode_from_dbn(
            wrap(
                &args,
                DynDecoder::inferred_with_buffer(io::stdin().lock(), args.upgrade_policy())?,
            ),
            &args,
        )
    } else {
        encode_from_dbn(
            wrap(
                &args,
                DynDecoder::from_file(&args.input, args.upgrade_policy())?,
            ),
            &args,
        )
    }
}
