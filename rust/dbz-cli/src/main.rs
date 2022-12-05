use std::io;

use clap::Parser;
use dbz::Dbz;
use dbz_cli::{infer_encoding, output_from_args, Args};

fn write_dbz<R: io::BufRead>(dbz: Dbz<R>, args: &Args) -> anyhow::Result<()> {
    let writer = output_from_args(args)?;
    let encoding = infer_encoding(args)?;
    if args.should_output_metadata {
        dbz.metadata().write_to(writer, encoding)?;
    } else {
        dbz.write_to(writer, encoding)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.input.as_os_str() == "-" {
        write_dbz(Dbz::new(io::stdin().lock())?, &args)
    } else {
        write_dbz(Dbz::from_file(&args.input)?, &args)
    }
}
