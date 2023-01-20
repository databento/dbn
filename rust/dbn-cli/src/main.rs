use std::io;

use clap::Parser;
use dbn::Dbn;
use dbn_cli::{infer_encoding, output_from_args, Args};

fn write_dbn<R: io::BufRead>(dbn: Dbn<R>, args: &Args) -> anyhow::Result<()> {
    let writer = output_from_args(args)?;
    let encoding = infer_encoding(args)?;
    if args.should_output_metadata {
        dbn.metadata().write_to(writer, encoding)?;
    } else {
        dbn.write_to(writer, encoding)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.input.as_os_str() == "-" {
        write_dbn(Dbn::new(io::stdin().lock())?, &args)
    } else {
        write_dbn(Dbn::from_file(&args.input)?, &args)
    }
}
