use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::Path,
};

use anyhow::{anyhow, Context};
use clap::Parser;
use dbn::{
    decode::{
        dbn::starts_with_prefix, DbnMetadata, DbnRecordDecoder, DecodeRecordRef, DynDecoder,
        DynReader, MergeDecoder, MergeRecordDecoder,
    },
    Compression, RecordHeaderKind, VersionUpgradePolicy,
};
use dbn_cli::{
    encode::{
        encode_from_dbn, encode_from_frag, silence_broken_pipe, split_encode_from_dbn,
        split_encode_from_frag, v4,
    },
    filter::{LimitFilter, SchemaFilter},
    Args,
};

const STDIN_SENTINEL: &str = "-";

fn open_input_file(path: &Path) -> anyhow::Result<File> {
    File::open(path).with_context(|| format!("opening file to decode at path '{}'", path.display()))
}

fn peek_input<R: BufRead>(
    reader: R,
) -> anyhow::Result<(BufReader<DynReader<'static, R>>, Option<u8>)> {
    let mut reader = BufReader::new(DynReader::inferred_with_buffer(reader)?);
    let prefix = reader.fill_buf()?;
    let version = starts_with_prefix(prefix).then(|| prefix[3]);
    Ok((reader, version))
}

fn output_pattern(args: &Args) -> anyhow::Result<&str> {
    args.output_pattern
        .as_deref()
        .ok_or_else(|| anyhow!("Must specify an output pattern when splitting files"))
}

fn wrap_frag(args: &Args, decoder: impl DecodeRecordRef) -> impl DecodeRecordRef {
    LimitFilter::new_no_metadata(
        SchemaFilter::new_no_metadata(decoder, args.schema_filter),
        args.limit,
    )
}

/// assume no ts_out for fragments
const FRAG_TS_OUT: bool = false;

fn decode_frag(
    args: &Args,
    upgrade_policy: VersionUpgradePolicy,
    reader: impl io::Read,
) -> anyhow::Result<impl DecodeRecordRef> {
    Ok(wrap_frag(
        args,
        DbnRecordDecoder::with_version(reader, args.input_version(), upgrade_policy, FRAG_TS_OUT)?,
    ))
}

fn wrap<H: RecordHeaderKind>(
    args: &Args,
    decoder: impl DecodeRecordRef<H> + DbnMetadata,
) -> impl DecodeRecordRef<H> + DbnMetadata {
    LimitFilter::new(SchemaFilter::new(decoder, args.schema_filter), args.limit)
}

fn encode_v4(
    args: &Args,
    decoder: impl DecodeRecordRef<dbn::v4::RecordHeader> + DbnMetadata,
) -> anyhow::Result<()> {
    if let Some(split_by) = args.split_by {
        v4::split_encode_from_dbn(args, split_by, output_pattern(args)?, decoder)
    } else {
        v4::encode_from_dbn(args, decoder, false)
    }
}

fn with_inputs(args: Args) -> anyhow::Result<()> {
    if let Some(compression) = args.input_fragment_compression() {
        return with_frag_inputs(&args, compression);
    }
    let inputs = args
        .input
        .iter()
        .map(|input| peek_input(BufReader::new(open_input_file(input)?)))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let input_versions = inputs
        .iter()
        .filter_map(|(_, version)| *version)
        .collect::<Vec<_>>();
    let upgrade_policy = args.upgrade_policy(&input_versions)?;
    if is_v4(upgrade_policy) {
        let decoders = inputs
            .into_iter()
            .map(|(reader, _)| v4::decoder(reader, upgrade_policy))
            .collect::<anyhow::Result<Vec<_>>>()?;
        v4::encode_from_dbn(&args, wrap(&args, MergeDecoder::new(decoders)?), true)
    } else {
        let decoders = inputs
            .into_iter()
            .map(|(reader, _)| DynDecoder::inferred_with_buffer(reader, upgrade_policy))
            .collect::<dbn::Result<Vec<_>>>()?;
        encode_from_dbn(&args, wrap(&args, MergeDecoder::new(decoders)?))
    }
}

fn with_frag_inputs(args: &Args, compression: Compression) -> anyhow::Result<()> {
    let readers = args
        .input
        .iter()
        .map(|input| Ok(DynReader::new(open_input_file(input)?, compression)?))
        .collect::<anyhow::Result<Vec<_>>>()?;
    let upgrade_policy = args.upgrade_policy(&[])?;
    if is_v4(upgrade_policy) {
        let decoders = readers
            .into_iter()
            .map(|reader| v4::frag_decoder(reader, args, FRAG_TS_OUT))
            .collect::<anyhow::Result<Vec<_>>>()?;
        let decoder = v4::FragDecoder::new(MergeRecordDecoder::new(decoders)?, FRAG_TS_OUT);
        v4::encode_from_dbn(args, wrap(args, decoder), true)
    } else {
        let decoders = readers
            .into_iter()
            .map(|reader| {
                DbnRecordDecoder::with_version(
                    reader,
                    args.input_version(),
                    upgrade_policy,
                    FRAG_TS_OUT,
                )
            })
            .collect::<dbn::Result<Vec<_>>>()?;
        encode_from_frag(args, MergeRecordDecoder::new(decoders)?)
    }
}

fn with_input(args: Args, reader: impl BufRead) -> anyhow::Result<()> {
    if let Some(compression) = args.input_fragment_compression() {
        return with_frag_input(&args, DynReader::with_buffer(reader, compression)?);
    }
    let (reader, version) = peek_input(reader)?;
    let upgrade_policy = args.upgrade_policy(version.as_slice())?;
    if is_v4(upgrade_policy) {
        return encode_v4(&args, wrap(&args, v4::decoder(reader, upgrade_policy)?));
    }
    let decoder = wrap(
        &args,
        DynDecoder::inferred_with_buffer(reader, upgrade_policy)?,
    );
    if let Some(split_by) = args.split_by {
        split_encode_from_dbn(&args, split_by, output_pattern(&args)?, decoder)
    } else {
        encode_from_dbn(&args, decoder)
    }
}

fn with_frag_input(args: &Args, reader: impl io::Read) -> anyhow::Result<()> {
    let upgrade_policy = args.upgrade_policy(&[])?;
    if is_v4(upgrade_policy) {
        let decoder =
            v4::FragDecoder::new(v4::frag_decoder(reader, args, FRAG_TS_OUT)?, FRAG_TS_OUT);
        return encode_v4(args, wrap(args, decoder));
    }
    let decoder = decode_frag(args, upgrade_policy, reader)?;
    if let Some(split_by) = args.split_by {
        split_encode_from_frag(args, split_by, output_pattern(args)?, decoder)
    } else {
        encode_from_frag(args, decoder)
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.input.len() > 1 {
        if args.split_by.is_some() {
            return Err(anyhow!("Can't split by files while merging files"));
        }
        with_inputs(args)
    } else if args.input[0].as_os_str() == STDIN_SENTINEL {
        with_input(args, io::stdin().lock())
    } else {
        let reader = BufReader::new(open_input_file(&args.input[0])?);
        with_input(args, reader)
    }
    .or_else(silence_broken_pipe)
}

fn is_v4(upgrade_policy: VersionUpgradePolicy) -> bool {
    matches!(
        upgrade_policy,
        VersionUpgradePolicy::UpgradeToV4 | VersionUpgradePolicy::ExpectV4
    )
}
