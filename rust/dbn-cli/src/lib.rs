use std::{
    fs::File,
    io::{self, BufWriter},
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use clap::{ArgAction, Parser, ValueEnum};

use dbn::{
    encode::{OutputEncoding, SplitDuration},
    enums::Compression,
    Schema, VersionUpgradePolicy,
};

pub mod encode;
pub mod filter;

/// How to split a DBN file
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SplitBy {
    Symbol,
    Schema,
    Day,
    Week,
    Month,
    Year,
}

impl SplitBy {
    pub fn duration(self) -> Option<SplitDuration> {
        match self {
            SplitBy::Day => Some(SplitDuration::Day),
            SplitBy::Week => Some(SplitDuration::Week),
            SplitBy::Month => Some(SplitDuration::Month),
            SplitBy::Year => Some(SplitDuration::Year),
            SplitBy::Symbol | SplitBy::Schema => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InferredEncoding {
    pub encoding: OutputEncoding,
    pub compression: Compression,
    pub delimiter: u8,
    pub is_fragment: bool,
}

#[derive(Debug, Parser)]
#[clap(name = "dbn", version, about)]
#[cfg_attr(test, derive(Default))]
pub struct Args {
    #[clap(
        help = "One or more DBN or legacy DBZ files to decode. Passing multiple files will result in a merge. Pass '-' to read from standard input",
        value_name = "FILE...",
        value_delimiter = ' ',
        num_args = 1..,
        required = true,
    )]
    pub input: Vec<PathBuf>,
    #[clap(
        short,
        long,
        help = "Saves the result to FILE. If no path is specified, the output will be written to standard output",
        value_name = "FILE"
    )]
    pub output: Option<PathBuf>,
    #[clap(
        short = 'O',
        long,
        help = "Saves the result of file splitting to paths according to PATTERN",
        requires = "split_by",
        conflicts_with = "output",
        value_name = "PATTERN"
    )]
    pub output_pattern: Option<String>,
    #[clap(
        short = 'S',
        long,
        help = "How to optionally split the output across files",
        requires = "output_pattern",
        value_name = "SPLIT_BY"
    )]
    pub split_by: Option<SplitBy>,
    #[clap(
        short = 'J',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as JSON lines"
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
        short = 'P',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as Apache Parquet"
    )]
    pub parquet: bool,
    #[clap(
        short = 'A',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as Apache Arrow IPC (Feather v2)"
    )]
    pub arrow: bool,
    #[clap(
        short = 'F',
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "output_encoding",
        help = "Output the result as a DBN fragment (no metadata)"
    )]
    pub fragment: bool,
    #[clap(
        short,
        long,
        action = ArgAction::SetTrue,
        default_value = "false",
        help = "Zstd compress the output"
    )]
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
        conflicts_with_all = ["csv", "tsv", "dbn", "fragment", "parquet", "arrow"],
        help = "Output the metadata section instead of the body of the DBN file. Only valid for JSON output encoding"
    )]
    pub should_output_metadata: bool,
    #[clap(
         short = 'p',
         long = "pretty",
         action = ArgAction::SetTrue,
         default_value = "false",
         conflicts_with_all = ["dbn", "fragment"],
         help ="Make the CSV, JSON, Parquet, or Arrow output easier to read by converting prices to decimals and, for CSV and JSON, timestamps to ISO 8601"
    )]
    pub should_pretty_print: bool,
    #[clap(
         short = 's',
         long = "map-symbols",
         action = ArgAction::SetTrue,
         default_value = "false",
         conflicts_with_all = ["input_fragment", "dbn", "fragment"],
         help ="Use symbology mappings from the metadata to create a 'symbol' field mapping the instrument ID to its requested symbol."
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
        conflicts_with_all = ["is_input_zstd_fragment", "should_output_metadata", "dbn", "parquet", "arrow"],
        help = "Interpret the input as an uncompressed DBN fragment, i.e. records without metadata. Only valid with text or DBN fragment output encodings"
    )]
    pub is_input_fragment: bool,
    #[clap(
        long = "input-zstd-fragment",
        action = ArgAction::SetTrue,
        default_value = "false",
        group = "input_fragment",
        conflicts_with_all = ["should_output_metadata", "dbn", "parquet", "arrow"],
        help = "Interpret the input as a Zstd-compressed DBN fragment, i.e. records without metadata. Only valid with text or DBN fragment output encodings"
    )]
    pub is_input_zstd_fragment: bool,
    #[clap(
        long = "input-dbn-version",
        help = "Specify the DBN version of the fragment. By default the fragment is assumed to be of the current version",
        value_name = "DBN_VERSION",
        value_parser = clap::value_parser!(u8).range(1..=3),
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
        conflicts_with_all = ["json", "dbn", "fragment", "parquet", "arrow"],
        help = "Skip encoding the header. Only valid when encoding CSV or TSV."
    )]
    pub write_header: bool,
}

impl Args {
    pub fn upgrade_policy(&self) -> VersionUpgradePolicy {
        if self.should_upgrade {
            VersionUpgradePolicy::UpgradeToV3
        } else {
            VersionUpgradePolicy::AsIs
        }
    }

    pub fn input_version(&self) -> u8 {
        self.input_dbn_version_override.unwrap_or(dbn::DBN_VERSION)
    }
}

pub fn infer_encoding(args: &Args) -> anyhow::Result<InferredEncoding> {
    let (format, zstd) = if let Some(format) = OUTPUT_FORMATS.iter().find(|f| (f.flag)(args)) {
        (format, args.zstd)
    } else {
        let Some(output) = args
            .output
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .or_else(|| args.output_pattern.clone())
        else {
            return Err(anyhow!(
                "Unable to infer output encoding when no output was specified"
            ));
        };
        let Some((format, extension)) = OUTPUT_FORMATS.iter().find_map(|f| {
            f.extensions
                .iter()
                .find(|ext| output.ends_with(*ext))
                .map(|ext| (f, ext))
        }) else {
            return Err(anyhow!(
                "Unable to infer output encoding from output path '{output}'",
            ));
        };
        (format, extension.ends_with(".zst"))
    };
    Ok(InferredEncoding {
        encoding: format.encoding,
        compression: if zstd {
            Compression::Zstd
        } else {
            Compression::None
        },
        delimiter: format.delimiter,
        is_fragment: format.is_fragment,
    })
}

/// Returns a writeable object where the `dbn` output will be directed.
pub fn output_from_args(args: &Args) -> anyhow::Result<Box<dyn io::Write + Send>> {
    output(args.output.as_deref(), args.force)
}

pub fn output(output: Option<&Path>, force: bool) -> anyhow::Result<Box<dyn io::Write + Send>> {
    if let Some(output) = output {
        let output_file = open_output_file(output, force)?;
        Ok(Box::new(BufWriter::new(output_file)))
    } else {
        // Wrap stdout in a BufWriter so each record write doesn't re-acquire
        // stdout's internal lock. `StdoutLock` isn't `Send`.
        Ok(Box::new(BufWriter::new(io::stdout())))
    }
}

/// An output format of the `dbn` command: the flag that selects it, how it's encoded,
/// and the output path extensions it's inferred from.
struct OutputFormat {
    flag: fn(&Args) -> bool,
    encoding: OutputEncoding,
    delimiter: u8,
    is_fragment: bool,
    /// Extensions ending in `.zst` infer Zstd compression.
    extensions: &'static [&'static str],
}

const OUTPUT_FORMATS: [OutputFormat; 7] = [
    OutputFormat {
        flag: |args| args.json,
        encoding: OutputEncoding::Json,
        delimiter: 0,
        is_fragment: false,
        extensions: &[".json", ".json.zst"],
    },
    OutputFormat {
        flag: |args| args.csv,
        encoding: OutputEncoding::Csv,
        delimiter: b',',
        is_fragment: false,
        extensions: &[".csv", ".csv.zst"],
    },
    OutputFormat {
        flag: |args| args.tsv,
        encoding: OutputEncoding::Csv,
        delimiter: b'\t',
        is_fragment: false,
        extensions: &[".tsv", ".xls", ".tsv.zst", ".xls.zst"],
    },
    OutputFormat {
        flag: |args| args.dbn,
        encoding: OutputEncoding::Dbn,
        delimiter: 0,
        is_fragment: false,
        extensions: &[".dbn", ".dbn.zst"],
    },
    OutputFormat {
        flag: |args| args.fragment,
        encoding: OutputEncoding::Dbn,
        delimiter: 0,
        is_fragment: true,
        extensions: &[".dbn.frag", ".dbn.frag.zst"],
    },
    OutputFormat {
        flag: |args| args.parquet,
        encoding: OutputEncoding::Parquet,
        delimiter: 0,
        is_fragment: false,
        extensions: &[".parquet"],
    },
    OutputFormat {
        flag: |args| args.arrow,
        encoding: OutputEncoding::Arrow,
        delimiter: 0,
        is_fragment: false,
        extensions: &[".arrow", ".feather"],
    },
];

fn open_output_file(path: &Path, force: bool) -> anyhow::Result<File> {
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
    #[case(
        true,
        false,
        false,
        false,
        false,
        OutputEncoding::Json,
        Compression::None,
        0
    )]
    #[case(
        false,
        true,
        false,
        false,
        false,
        OutputEncoding::Csv,
        Compression::None,
        b','
    )]
    #[case(
        false,
        false,
        true,
        false,
        false,
        OutputEncoding::Csv,
        Compression::None,
        b'\t'
    )]
    #[case(
        false,
        false,
        false,
        true,
        false,
        OutputEncoding::Dbn,
        Compression::None,
        0
    )]
    #[case(
        true,
        false,
        false,
        false,
        true,
        OutputEncoding::Json,
        Compression::Zstd,
        0
    )]
    #[case(
        false,
        true,
        false,
        false,
        true,
        OutputEncoding::Csv,
        Compression::Zstd,
        b','
    )]
    #[case(
        false,
        false,
        true,
        false,
        true,
        OutputEncoding::Csv,
        Compression::Zstd,
        b'\t'
    )]
    #[case(
        false,
        false,
        false,
        true,
        true,
        OutputEncoding::Dbn,
        Compression::Zstd,
        0
    )]
    fn test_infer_encoding_and_compression_explicit(
        #[case] json: bool,
        #[case] csv: bool,
        #[case] tsv: bool,
        #[case] dbn: bool,
        #[case] zstd: bool,
        #[case] exp_enc: OutputEncoding,
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
        assert_eq!(
            infer_encoding(&args).unwrap(),
            InferredEncoding {
                encoding: exp_enc,
                compression: exp_comp,
                delimiter: exp_sep,
                is_fragment: false,
            }
        );
    }

    #[rstest]
    #[case("out.json", OutputEncoding::Json, Compression::None, 0)]
    #[case("out.csv", OutputEncoding::Csv, Compression::None, b',')]
    #[case("out.tsv", OutputEncoding::Csv, Compression::None, b'\t')]
    #[case("out.xls", OutputEncoding::Csv, Compression::None, b'\t')]
    #[case("out.dbn", OutputEncoding::Dbn, Compression::None, 0)]
    #[case("out.json.zst", OutputEncoding::Json, Compression::Zstd, 0)]
    #[case("out.csv.zst", OutputEncoding::Csv, Compression::Zstd, b',')]
    #[case("out.tsv.zst", OutputEncoding::Csv, Compression::Zstd, b'\t')]
    #[case("out.xls.zst", OutputEncoding::Csv, Compression::Zstd, b'\t')]
    #[case("out.dbn.zst", OutputEncoding::Dbn, Compression::Zstd, 0)]
    fn test_infer_encoding_and_compression_inference(
        #[case] output: &str,
        #[case] exp_enc: OutputEncoding,
        #[case] exp_comp: Compression,
        #[case] exp_sep: u8,
    ) {
        let args = Args {
            output: Some(PathBuf::from(output)),
            ..Default::default()
        };
        assert_eq!(
            infer_encoding(&args).unwrap(),
            InferredEncoding {
                encoding: exp_enc,
                compression: exp_comp,
                delimiter: exp_sep,
                is_fragment: false,
            }
        );
    }

    #[rstest]
    #[case("out.dbn.frag", OutputEncoding::Dbn, Compression::None, true)]
    #[case("out.dbn.frag.zst", OutputEncoding::Dbn, Compression::Zstd, true)]
    #[case("out.parquet", OutputEncoding::Parquet, Compression::None, false)]
    #[case("out.arrow", OutputEncoding::Arrow, Compression::None, false)]
    #[case("out.feather", OutputEncoding::Arrow, Compression::None, false)]
    fn test_infer_encoding_fragment_and_columnar(
        #[case] output: &str,
        #[case] exp_enc: OutputEncoding,
        #[case] exp_comp: Compression,
        #[case] exp_fragment: bool,
    ) {
        let args = Args {
            output: Some(PathBuf::from(output)),
            ..Default::default()
        };
        assert_eq!(
            infer_encoding(&args).unwrap(),
            InferredEncoding {
                encoding: exp_enc,
                compression: exp_comp,
                delimiter: 0,
                is_fragment: exp_fragment,
            }
        );
    }

    #[rstest]
    #[case::unknown("out.pb")]
    #[case::columnar_zstd("out.parquet.zst")]
    #[case::columnar_zstd("out.arrow.zst")]
    fn test_infer_encoding_and_compression_bad(#[case] output: &str) {
        let args = Args {
            output: Some(PathBuf::from(output)),
            ..Default::default()
        };
        assert!(
            matches!(infer_encoding(&args), Err(e) if e.to_string().starts_with("Unable to infer"))
        );
    }
}
