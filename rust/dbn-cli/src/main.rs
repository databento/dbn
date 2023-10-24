use std::{fs::File, io};

use clap::Parser;
use dbn::decode::{DbnRecordDecoder, DynDecoder};
use dbn_cli::{
    encode::{encode_from_dbn, encode_from_frag},
    Args,
};

const STDIN_SENTINEL: &str = "-";

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let input_version = args.input_dbn_version_override.unwrap_or(dbn::DBN_VERSION);
    if args.is_input_fragment {
        if args.input.as_os_str() == STDIN_SENTINEL {
            encode_from_frag(
                DbnRecordDecoder::with_version(
                    io::stdin().lock(),
                    input_version,
                    args.upgrade_policy(),
                )?,
                &args,
            )
        } else {
            encode_from_frag(
                DbnRecordDecoder::with_version(
                    File::open(args.input.clone())?,
                    input_version,
                    args.upgrade_policy(),
                )?,
                &args,
            )
        }
    } else if args.is_input_zstd_fragment {
        if args.input.as_os_str() == STDIN_SENTINEL {
            encode_from_frag(
                DbnRecordDecoder::with_version(
                    zstd::stream::Decoder::with_buffer(io::stdin().lock())?,
                    input_version,
                    args.upgrade_policy(),
                )?,
                &args,
            )
        } else {
            encode_from_frag(
                DbnRecordDecoder::with_version(
                    zstd::stream::Decoder::new(File::open(args.input.clone())?)?,
                    input_version,
                    args.upgrade_policy(),
                )?,
                &args,
            )
        }
    } else if args.input.as_os_str() == STDIN_SENTINEL {
        encode_from_dbn(
            DynDecoder::inferred_with_buffer(io::stdin().lock(), args.upgrade_policy())?,
            &args,
        )
    } else {
        encode_from_dbn(
            DynDecoder::from_file(&args.input, args.upgrade_policy())?,
            &args,
        )
    }
}
