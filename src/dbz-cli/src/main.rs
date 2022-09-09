use clap::Parser;
use dbz_cli::{infer_encoding, output_from_args, Args};
use dbz_lib::Dbz;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let dbz = Dbz::from_file(&args.input)?;
    let writer = output_from_args(&args)?;
    let encoding = infer_encoding(&args)?;
    if args.should_output_metadata {
        dbz.metadata().write_to(writer, encoding)?;
    } else {
        dbz.write_to(writer, encoding)?;
    }
    Ok(())
}
