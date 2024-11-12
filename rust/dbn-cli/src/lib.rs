use std::{
    fs::File,
    io::{self, BufWriter},
    num::NonZeroU64,
    path::PathBuf,
};

use anyhow::{anyhow, Context};
use clap::{ArgAction, Parser, ValueEnum};

use dbn::{
    enums::{Compression, Encoding},
    Schema, VersionUpgradePolicy,
};

pub mod encode;
pub mod filter;

/// How the output of the `dbn` command will be encoded.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OutputEncoding {
    /// `dbn` will infer based on the extension of the specified output file
    Infer,
    Dbn,
    Csv,
    Tsv,
    Json,
    DbnFragment,
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
        group = "output_encoding",
        help = "Output the result as NDJSON (newline-delimited JSON)"
    )]
    pub json: bool,
    #[clap(
        short = 'C',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as CSV"
    )]
    pub csv: bool,
    #[clap(
        short = 'T',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as tab-separated values (TSV)"
    )]
    pub tsv: bool,
    #[clap(
        short = 'D',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as DBN"
    )]
    pub dbn: bool,
    #[clap(
        short = 'F',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as a DBN fragment (no metadata)"
    )]
    pub fragment: bool,
    #[clap(short, long, action = ArgAction::SetTrue, default_value = "false", help = "Zstd compress the output")]
    pub zstd: bool,
    #[clap(
        short = 'u',
        long = "upgrade",
        default_value = "false",
        action = ArgAction::SetTrue,
        help = "Upgrade data when decoding previous DBN versions. By default data is decoded as-is."
    )]
    pub should_upgrade: bool,
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
        conflicts_with_all = ["csv", "dbn", "fragment"],
        help = "Output the metadata section instead of the body of the DBN file. Only valid for JSON output encoding"
    )]
    pub should_output_metadata: bool,
    #[clap(
         short = 'p',
         long = "pretty",
         action = ArgAction::SetTrue,
         default_value = "false",
         conflicts_with_all = ["dbn", "fragment"],
         help ="Make the CSV or JSON output easier to read by converting timestamps to ISO 8601 and prices to decimals"
    )]
    pub should_pretty_print: bool,
    #[clap(
         short = 's',
         long = "map-symbols",
         action = ArgAction::SetTrue,
         default_value = "false",
         conflicts_with_all = ["input_fragment", "dbn", "fragment"],
         help ="Use symbology mappings from the metadata to create a 'symbol' field mapping the intstrument ID to its requested symbol."
    )]
    pub map_symbols: bool,
    #[clap(
        short = 'l',
        long = "limit",
        value_name = "NUM_RECORDS",
        help = "Limit the number of records in the output to the specified number"
    )]
    pub limit: Option<NonZeroU64>,
    // Fragment arguments
    #[clap(
        long = "input-fragment",
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "input_fragment",
        conflicts_with_all = ["is_input_zstd_fragment", "should_output_metadata", "dbn"],
        help = "Interpret the input as an uncompressed DBN fragment, i.e. records without metadata. Only valid with text output encodings"
    )]
    pub is_input_fragment: bool,
    #[clap(
        long = "input-zstd-fragment",
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "input_fragment",
        conflicts_with_all = ["should_output_metadata", "dbn"],
        help = "Interpret the input as a Zstd-compressed DBN fragment, i.e. records without metadata. Only valid with text output encodings"
    )]
    pub is_input_zstd_fragment: bool,
    #[clap(
        long = "input-dbn-version",
        help = "Specify the DBN version of the fragment. By default the fragment is assumed to be of the current version",
        value_name = "DBN_VERSION",
        value_parser = clap::value_parser!(u8).range(1..=2),
        requires = "input_fragment"
    )]
    pub input_dbn_version_override: Option<u8>,
    #[clap(
        long = "schema",
        help = "Only encode records of this schema. This is particularly useful for transcoding mixed-schema DBN to CSV, which doesn't support mixing schemas",
        value_name = "SCHEMA"
    )]
    pub schema_filter: Option<Schema>,
    #[clap(
        long = "omit-header",
        action = ArgAction::SetFalse,
        default_value = "true",
        conflicts_with_all = ["json", "dbn", "fragment"],
        help = "Skip encoding the header. Only valid when encoding CSV or TSV."
    )]
    pub write_header: bool,
}

impl Args {
    /// Consolidates the several output flag booleans into a single enum.
    pub fn output_encoding(&self) -> OutputEncoding {
        if self.json {
            OutputEncoding::Json
        } else if self.csv {
            OutputEncoding::Csv
        } else if self.tsv {
            OutputEncoding::Tsv
        } else if self.dbn {
            OutputEncoding::Dbn
        } else if self.fragment {
            OutputEncoding::DbnFragment
        } else {
            OutputEncoding::Infer
        }
    }

    pub fn upgrade_policy(&self) -> VersionUpgradePolicy {
        if self.should_upgrade {
            VersionUpgradePolicy::Upgrade
        } else {
            VersionUpgradePolicy::AsIs
        }
    }

    pub fn input_version(&self) -> u8 {
        self.input_dbn_version_override.unwrap_or(dbn::DBN_VERSION)
    }
}

/// Infer the [`Encoding`], [`Compression`], and delimiter (CSV/TSV) from `args` if they
/// aren't already explicitly set.
pub fn infer_encoding(args: &Args) -> anyhow::Result<(Encoding, Compression, u8)> {
    let compression = if args.zstd {
        Compression::ZStd
    } else {
        Compression::None
    };
    match args.output_encoding() {
        OutputEncoding::DbnFragment | OutputEncoding::Dbn => Ok((Encoding::Dbn, compression, 0)),
        OutputEncoding::Csv => Ok((Encoding::Csv, compression, b',')),
        OutputEncoding::Tsv => Ok((Encoding::Csv, compression, b'\t')),
        OutputEncoding::Json => Ok((Encoding::Json, compression, 0)),
        OutputEncoding::Infer => {
            if let Some(output) = args.output.as_ref().map(|o| o.to_string_lossy()) {
                if output.ends_with(".dbn.zst") {
                    Ok((Encoding::Dbn, Compression::ZStd, 0))
                } else if output.ends_with(".dbn") {
                    Ok((Encoding::Dbn, Compression::None, 0))
                } else if output.ends_with(".csv.zst") {
                    Ok((Encoding::Csv, Compression::ZStd, b','))
                } else if output.ends_with(".csv") {
                    Ok((Encoding::Csv, Compression::None, b','))
                } else if output.ends_with(".tsv.zst") || output.ends_with(".xls.zst") {
                    Ok((Encoding::Csv, Compression::ZStd, b'\t'))
                } else if output.ends_with(".tsv") || output.ends_with(".xls") {
                    Ok((Encoding::Csv, Compression::None, b'\t'))
                } else if output.ends_with(".json.zst") {
                    Ok((Encoding::Json, Compression::ZStd, 0))
                } else if output.ends_with(".json") {
                    Ok((Encoding::Json, Compression::None, 0))
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
    options.write(true).truncate(true);
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
    #![allow(clippy::too_many_arguments)]

    use rstest::*;

    use super::*;

    #[rstest]
    #[case(true, false, false, false, false, Encoding::Json, Compression::None, 0)]
    #[case(
        false,
        true,
        false,
        false,
        false,
        Encoding::Csv,
        Compression::None,
        b','
    )]
    #[case(
        false,
        false,
        true,
        false,
        false,
        Encoding::Csv,
        Compression::None,
        b'\t'
    )]
    #[case(false, false, false, true, false, Encoding::Dbn, Compression::None, 0)]
    #[case(true, false, false, false, true, Encoding::Json, Compression::ZStd, 0)]
    #[case(
        false,
        true,
        false,
        false,
        true,
        Encoding::Csv,
        Compression::ZStd,
        b','
    )]
    #[case(
        false,
        false,
        true,
        false,
        true,
        Encoding::Csv,
        Compression::ZStd,
        b'\t'
    )]
    #[case(false, false, false, true, true, Encoding::Dbn, Compression::ZStd, 0)]
    fn test_infer_encoding_and_compression_explicit(
        #[case] json: bool,
        #[case] csv: bool,
        #[case] tsv: bool,
        #[case] dbn: bool,
        #[case] zstd: bool,
        #[case] exp_enc: Encoding,
        #[case] exp_comp: Compression,
        #[case] exp_sep: u8,
    ) {
        let args = Args {
            json,
            csv,
            tsv,
            dbn,
            zstd,
            ..Default::default()
        };
        assert_eq!(infer_encoding(&args).unwrap(), (exp_enc, exp_comp, exp_sep));
    }

    #[rstest]
    #[case("out.json", Encoding::Json, Compression::None, 0)]
    #[case("out.csv", Encoding::Csv, Compression::None, b',')]
    #[case("out.tsv", Encoding::Csv, Compression::None, b'\t')]
    #[case("out.xls", Encoding::Csv, Compression::None, b'\t')]
    #[case("out.dbn", Encoding::Dbn, Compression::None, 0)]
    #[case("out.json.zst", Encoding::Json, Compression::ZStd, 0)]
    #[case("out.csv.zst", Encoding::Csv, Compression::ZStd, b',')]
    #[case("out.tsv.zst", Encoding::Csv, Compression::ZStd, b'\t')]
    #[case("out.xls.zst", Encoding::Csv, Compression::ZStd, b'\t')]
    #[case("out.dbn.zst", Encoding::Dbn, Compression::ZStd, 0)]
    fn test_infer_encoding_and_compression_inference(
        #[case] output: &str,
        #[case] exp_enc: Encoding,
        #[case] exp_comp: Compression,
        #[case] exp_sep: u8,
    ) {
        let args = Args {
            output: Some(PathBuf::from(output)),
            ..Default::default()
        };
        assert_eq!(infer_encoding(&args).unwrap(), (exp_enc, exp_comp, exp_sep));
    }

    #[test]
    fn test_infer_encoding_and_compression_bad() {
        let args = Args {
            output: Some(PathBuf::from("out.pb")),
            ..Default::default()
        };
        assert!(
            matches!(infer_encoding(&args), Err(e) if e.to_string().starts_with("Unable to infer"))
        );
    }
}
