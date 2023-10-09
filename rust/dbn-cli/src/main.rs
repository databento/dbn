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
    if args.is_input_fragment {
        if args.input.as_os_str() == STDIN_SENTINEL {
            encode_from_frag(DbnRecordDecoder::new(io::stdin().lock()), &args)
        } else {
            encode_from_frag(
                DbnRecordDecoder::new(File::open(args.input.clone())?),
                &args,
            )
        }
    } else if args.is_input_zstd_fragment {
        if args.input.as_os_str() == STDIN_SENTINEL {
            encode_from_frag(
                DbnRecordDecoder::new(zstd::stream::Decoder::with_buffer(io::stdin().lock())?),
                &args,
            )
        } else {
            encode_from_frag(
                DbnRecordDecoder::new(zstd::stream::Decoder::new(File::open(args.input.clone())?)?),
                &args,
            )
        }
    } else if args.input.as_os_str() == "-" {
        encode_from_dbn(DynDecoder::inferred_with_buffer(io::stdin().lock())?, &args)
    } else {
        encode_from_dbn(DynDecoder::from_file(&args.input)?, &args)
    }
}
