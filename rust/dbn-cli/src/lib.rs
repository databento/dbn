use std::{
    fs::File,
    io::{self, BufWriter},
    path::PathBuf,
};

use anyhow::{anyhow, Context};
use clap::{ArgAction, Parser, ValueEnum};

use dbn::enums::{Compression, Encoding};

/// How the output of the `dbn` command will be encoded.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputEncoding {
    /// `dbn` will infer based on the extension of the specified output file
    Infer,
    Dbn,
    Csv,
    Json,
}

#[derive(Debug, Parser)]
#[clap(version, about)]
#[cfg_attr(test, derive(Default))]
pub struct Args {
    #[clap(
        help = "A DBN or legacy DBZ file to convert to another encoding. Pass '-' to read from standard input",
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
        short = 'D',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        conflicts_with = "json",
        help = "Output the result as DBN"
    )]
    pub dbn: bool,
    #[clap(short, long, action = ArgAction::SetTrue, default_value = "false", help = "Zstd compress the output")]
    pub zstd: bool,
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
        conflicts_with = "csv",
        conflicts_with = "dbn",
        help = "Output the metadata section instead of the body of the DBN file. Only valid for JSON output encoding"
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
    /// Consolidates the several output flag booleans into a single enum.
    pub fn output_encoding(&self) -> OutputEncoding {
        if self.json {
            OutputEncoding::Json
        } else if self.csv {
            OutputEncoding::Csv
        } else if self.dbn {
            OutputEncoding::Dbn
        } else {
            OutputEncoding::Infer
        }
    }
}

/// Infer the [`Encoding`] and [`Compression`] from `args` if they aren't already explicitly
/// set.
pub fn infer_encoding_and_compression(args: &Args) -> anyhow::Result<(Encoding, Compression)> {
    let compression = if args.zstd {
        Compression::ZStd
    } else {
        Compression::None
    };
    match args.output_encoding() {
        OutputEncoding::Dbn => Ok((Encoding::Dbn, compression)),
        OutputEncoding::Csv => Ok((Encoding::Csv, compression)),
        OutputEncoding::Json => Ok((Encoding::Json, compression)),
        OutputEncoding::Infer => {
            if let Some(output) = args.output.as_ref().map(|o| o.to_string_lossy()) {
                if output.ends_with(".dbn.zst") {
                    Ok((Encoding::Dbn, Compression::ZStd))
                } else if output.ends_with(".dbn") {
                    Ok((Encoding::Dbn, Compression::None))
                } else if output.ends_with(".csv.zst") {
                    Ok((Encoding::Csv, Compression::ZStd))
                } else if output.ends_with(".csv") {
                    Ok((Encoding::Csv, Compression::None))
                } else if output.ends_with(".json.zst") {
                    Ok((Encoding::Json, Compression::ZStd))
                } else if output.ends_with(".json") {
                    Ok((Encoding::Json, Compression::None))
                } else {
                    Err(anyhow!(
                        "Unable to infer output encoding from output path '{output}'",
                    ))
                }
            } else {
                Err(anyhow!(
                    "Unable to infer output encoding when no output was specified"
                ))
            }
        }
    }
}

/// Returns a writeable object where the `dbn` output will be directed.
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
    options.truncate(true);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_encoding_and_compression_explicit() {
        let combinations = [
            (true, false, false, false, Encoding::Json, Compression::None),
            (false, true, false, false, Encoding::Csv, Compression::None),
            (false, false, true, false, Encoding::Dbn, Compression::None),
            (true, false, false, true, Encoding::Json, Compression::ZStd),
            (false, true, false, true, Encoding::Csv, Compression::ZStd),
            (false, false, true, true, Encoding::Dbn, Compression::ZStd),
        ];
        for (json, csv, dbn, zstd, exp_enc, exp_comp) in combinations {
            let args = Args {
                json,
                csv,
                dbn,
                zstd,
                ..Default::default()
            };
            assert_eq!(
                infer_encoding_and_compression(&args).unwrap(),
                (exp_enc, exp_comp)
            );
        }
    }

    #[test]
    fn test_infer_encoding_and_compression_inference() {
        let combinations = [
            ("out.json", Encoding::Json, Compression::None),
            ("out.csv", Encoding::Csv, Compression::None),
            ("out.dbn", Encoding::Dbn, Compression::None),
            ("out.json.zst", Encoding::Json, Compression::ZStd),
            ("out.csv.zst", Encoding::Csv, Compression::ZStd),
            ("out.dbn.zst", Encoding::Dbn, Compression::ZStd),
        ];
        for (output, exp_enc, exp_comp) in combinations {
            let args = Args {
                output: Some(PathBuf::from(output)),
                ..Default::default()
            };
            assert_eq!(
                infer_encoding_and_compression(&args).unwrap(),
                (exp_enc, exp_comp)
            );
        }
    }

    #[test]
    fn test_infer_encoding_and_compression_bad() {
        let args = Args {
            output: Some(PathBuf::from("out.pb")),
            ..Default::default()
        };
        assert!(
            matches!(infer_encoding_and_compression(&args), Err(e) if e.to_string().starts_with("Unable to infer"))
        );
    }
}
