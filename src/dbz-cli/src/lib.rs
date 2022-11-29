use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};

use anyhow::{anyhow, Context};
use clap::{ArgAction, Parser, ValueEnum};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputEncoding {
    /// `dbz` will infer based on the extension of the specified output file
    Infer,
    Csv,
    Json,
}

#[derive(Debug, Parser)]
#[clap(version, about)]
pub struct Args {
    #[clap(
        help = "A DBZ file to convert to another encoding. Pass '-' to read from standard input",
        value_name = "FILE"
    )]
    pub input: PathBuf,
    #[clap(
        short,
        long,
        help = "Saves the result to FILE. If no path is specified, the output will be written to standard output",
        value_name = "FILE"
    )]
    pub output: Option<PathBuf>,
    #[clap(
        short = 'J',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        help = "Output the result as NDJSON (newline-delimited JSON)"
    )]
    pub json: bool,
    #[clap(
        short = 'C',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        conflicts_with = "json",
        help = "Output the result as CSV"
    )]
    pub csv: bool,
    #[clap(
        short,
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        help = "Allow overwriting of existing files, such as the output file"
    )]
    pub force: bool,
    #[clap(
        short = 'm',
        long = "metadata",
        action = ArgAction::SetTrue,
        default_value = "false",
        help = "Output the metadata section instead of the body of the DBZ file"
    )]
    pub should_output_metadata: bool,
    #[clap(
         short = 'p',
         long = "pretty-json",
         action = ArgAction::SetTrue,
         default_value = "false",
         help ="Make the JSON output easier to read with spacing and indentation"
    )]
    pub should_pretty_print: bool,
}

impl Args {
    pub fn output_encoding(&self) -> OutputEncoding {
        match (self.json, self.csv) {
            (false, false) => OutputEncoding::Infer,
            (true, false) => OutputEncoding::Json,
            (false, true) => OutputEncoding::Csv,
            (true, true) => unreachable!("Invalid state that clap conflicts_with should prevent"),
        }
    }
}

pub fn infer_encoding(args: &Args) -> anyhow::Result<dbz_lib::OutputEncoding> {
    match args.output_encoding() {
        OutputEncoding::Csv => Ok(dbz_lib::OutputEncoding::Csv),
        OutputEncoding::Json => Ok(dbz_lib::OutputEncoding::Json {
            should_pretty_print: args.should_pretty_print,
        }),
        OutputEncoding::Infer => match args.output.as_ref().and_then(|o| o.extension()) {
            Some(ext) if ext == "csv" => Ok(dbz_lib::OutputEncoding::Csv),
            Some(ext) if ext == "json" => Ok(dbz_lib::OutputEncoding::Json {
                should_pretty_print: args.should_pretty_print,
            }),
            Some(ext) => Err(anyhow!(
                "Unable to infer output encoding from output file with extension '{}'",
                ext.to_string_lossy()
            )),
            None => Err(anyhow!(
                "Unable to infer output encoding from output file without an extension"
            )),
        },
    }
}

pub fn output_from_args(args: &Args) -> anyhow::Result<Box<dyn io::Write>> {
    if let Some(output) = &args.output {
        let output_file = open_output_file(output, args.force)?;
        Ok(Box::new(BufWriter::new(output_file)))
    } else {
        Ok(Box::new(io::stdout().lock()))
    }
}

fn open_output_file(path: &PathBuf, force: bool) -> anyhow::Result<File> {
    let mut options = File::options();
    options.write(true);
    if force {
        options.create(true);
    } else if path.exists() {
        return Err(anyhow!(
            "Output file exists. Pass --force flag to overwrite the existing file."
        ));
    } else {
        options.create_new(true);
    }
    options
        .open(path)
        .with_context(|| format!("Unable to open output file '{}'", path.display()))
}
