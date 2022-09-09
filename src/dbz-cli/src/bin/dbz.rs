use clap::Parser;
use dbz_cli::{infer_encoding, output_from_args, Args};
use dbz_lib::Dbz;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dbz = Dbz::from_file(&args.input)?;
    let writer = output_from_args(&args)?;
    let encoding = infer_encoding(&args)?;
    dbz.write_to(writer, encoding)?;
    Ok(())
}
