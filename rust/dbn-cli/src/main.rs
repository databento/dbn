use std::io;

use clap::Parser;
use dbn::{
    decode::{DecodeDbn, DynDecoder},
    encode::{json, DynEncoder, EncodeDbn},
};
use dbn_cli::{infer_encoding_and_compression, output_from_args, Args};

fn write_dbn<R: io::BufRead>(decoder: DynDecoder<R>, args: &Args) -> anyhow::Result<()> {
    let writer = output_from_args(args)?;
    let (encoding, compression) = infer_encoding_and_compression(args)?;
    if args.should_output_metadata {
        assert!(args.json);
        json::Encoder::new(writer, args.should_pretty_print).encode_metadata(decoder.metadata())
    } else {
        DynEncoder::new(
            writer,
            encoding,
            compression,
            decoder.metadata(),
            args.should_pretty_print,
        )?
        .encode_decoded(decoder)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.input.as_os_str() == "-" {
        write_dbn(DynDecoder::inferred_with_buffer(io::stdin().lock())?, &args)
    } else {
        write_dbn(DynDecoder::from_file(&args.input)?, &args)
    }
}
