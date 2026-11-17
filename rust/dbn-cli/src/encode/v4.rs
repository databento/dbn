use std::{
    io::{self, Write},
    path::Path,
};

use anyhow::anyhow;
use dbn::{
    decode::{
        dbn::{fsm::DbnFsm, v4::Decoder, RecordDecoder},
        DbnMetadata, DecodeRecordRef,
    },
    encode::{
        csv, dbn as dbn_encode, json, DynWriter, NoSchemaBehavior, OutputEncoding, SchemaSplitter,
        Splitter, TimeSplitter,
    },
    v4::{self, framing::is_framing_rtype},
    Metadata, Schema, SymbolIndex, TsSymbolMap, VersionUpgradePolicy,
};

use crate::{infer_encoding, output_from_args, Args, InferredEncoding, SplitBy};

type Writer = DynWriter<'static, Box<dyn io::Write + Send>>;

#[allow(
    clippy::large_enum_variant,
    reason = "a run builds few encoders, one per output file"
)]
enum Encoder {
    Dbn(dbn_encode::v4::Encoder<Writer>),
    Csv(csv::v4::Encoder<Writer>),
    Json(json::v4::Encoder<Writer>),
}

pub struct FragDecoder<D> {
    decoder: D,
    metadata: Metadata,
}

pub fn decoder<R: io::Read>(
    reader: R,
    upgrade_policy: VersionUpgradePolicy,
) -> anyhow::Result<Decoder<R>> {
    let fsm = DbnFsm::builder()
        .header::<v4::RecordHeader>()
        .upgrade_policy(upgrade_policy)
        .skip_framing_records(false)
        .build()?;
    Ok(Decoder::with_fsm(reader, fsm)?)
}

pub fn frag_decoder<R: io::Read>(
    reader: R,
    args: &Args,
    ts_out: bool,
) -> anyhow::Result<RecordDecoder<R, v4::RecordHeader>> {
    let fsm = DbnFsm::builder()
        .header::<v4::RecordHeader>()
        .skip_metadata(true)
        .input_dbn_version(Some(args.input_version()))?
        .upgrade_policy(VersionUpgradePolicy::UpgradeToV4)
        .ts_out(ts_out)
        .build()?;
    Ok(RecordDecoder::with_fsm(reader, fsm))
}

pub fn encode_from_dbn<D>(args: &Args, mut decoder: D, is_merged: bool) -> anyhow::Result<()>
where
    D: DecodeRecordRef<v4::RecordHeader> + DbnMetadata,
{
    let writer = output_from_args(args)?;
    let inferred = infer_encoding(args)?;
    if args.should_output_metadata {
        return super::encode_metadata(args, inferred.encoding, writer, decoder.metadata());
    }
    let symbol_map = symbol_map(args, decoder.metadata())?;
    let mut encoder = Encoder::new(writer, inferred, args, decoder.metadata())?;
    while let Some(record) = decoder.decode_record_ref()? {
        if is_framing_rtype(record.header().rtype) {
            check_record_group(args, !is_merged)?;
            encoder.encode_framing(record)?;
        } else {
            encoder.encode(record, symbol_map.as_ref())?;
        }
    }
    encoder.finish()?;
    Ok(())
}

pub fn split_encode_from_dbn<D>(
    args: &Args,
    split_by: SplitBy,
    output_pattern: &str,
    decoder: D,
) -> anyhow::Result<()>
where
    D: DecodeRecordRef<v4::RecordHeader> + DbnMetadata,
{
    let inferred = infer_encoding(args)?;
    let build_encoder = |path: &str, metadata: Option<Metadata>| -> dbn::Result<_> {
        let writer = crate::output(Some(Path::new(path)), args.force).map_err(|e| {
            dbn::Error::io(io::Error::other(e), format!("opening output file {path}"))
        })?;
        Encoder::new(writer, inferred, args, &metadata.unwrap())
    };
    match split_by {
        SplitBy::Symbol => Err(anyhow!(
            "Splitting DBN version 4 by symbol isn't supported yet"
        )),
        SplitBy::Schema => split_encode_impl(
            args,
            decoder,
            SchemaSplitter::new(
                |schema: Schema, metadata| {
                    build_encoder(
                        &output_pattern.replace("{schema}", schema.as_str()),
                        metadata,
                    )
                },
                NoSchemaBehavior::Skip,
            ),
        ),
        SplitBy::Day | SplitBy::Week | SplitBy::Month | SplitBy::Year => split_encode_impl(
            args,
            decoder,
            TimeSplitter::new(
                |date: time::Date, metadata| {
                    build_encoder(
                        &output_pattern.replace("{date}", &date.to_string()),
                        metadata,
                    )
                },
                split_by.duration().unwrap(),
            ),
        ),
    }
}

impl Encoder {
    fn new(
        writer: Box<dyn io::Write + Send>,
        inferred: InferredEncoding,
        args: &Args,
        metadata: &Metadata,
    ) -> dbn::Result<Self> {
        if inferred.is_fragment {
            return Err(dbn::Error::encode(
                "DBN version 4 fragment output isn't supported yet",
            ));
        }
        let writer = DynWriter::new(writer, inferred.compression)?;
        let pretty = args.should_pretty_print;
        match inferred.encoding {
            OutputEncoding::Dbn => dbn_encode::v4::Encoder::new(writer, metadata).map(Self::Dbn),
            OutputEncoding::Csv => csv::v4::Encoder::builder(writer, metadata)
                .delimiter(inferred.delimiter)
                .write_header(args.write_header)
                .use_pretty_px(pretty)
                .use_pretty_ts(pretty)
                .with_symbol(args.map_symbols)
                .build()
                .map(Self::Csv),
            OutputEncoding::Json => {
                json::v4::Encoder::new(writer, metadata, pretty, pretty, pretty).map(Self::Json)
            }
            _ => Err(dbn::Error::encode(
                "Parquet and Arrow output don't support DBN version 4 yet",
            )),
        }
    }

    fn encode(
        &mut self,
        record: v4::RecordRef,
        symbol_map: Option<&TsSymbolMap>,
    ) -> dbn::Result<()> {
        match (self, symbol_map) {
            (Self::Dbn(encoder), _) => encoder.encode_ref(record),
            (Self::Csv(encoder), Some(symbol_map)) => encoder
                .encode_ref_with_sym(record, symbol_map.get_for_rec(&record).map(String::as_str)),
            (Self::Csv(encoder), None) => encoder.encode_record_ref(record),
            (Self::Json(encoder), Some(symbol_map)) => encoder
                .encode_ref_with_sym(record, symbol_map.get_for_rec(&record).map(String::as_str)),
            (Self::Json(encoder), None) => encoder.encode_record_ref(record),
        }
    }

    fn encode_framing(&mut self, record: v4::RecordRef) -> dbn::Result<()> {
        if let Self::Dbn(encoder) = self {
            encoder
                .get_mut()
                .write_all(record.as_ref())
                .map_err(|e| dbn::Error::io(e, "passing through framing record"))?;
        }
        Ok(())
    }

    fn finish(self) -> dbn::Result<()> {
        match self {
            Self::Dbn(mut encoder) => encoder.get_mut().finish(),
            Self::Csv(encoder) => encoder.into_inner()?.finish(),
            Self::Json(mut encoder) => {
                encoder.flush()?;
                encoder.get_mut().finish()
            }
        }
    }
}

impl<D> FragDecoder<D> {
    pub fn new(decoder: D, ts_out: bool) -> Self {
        let mut metadata = super::dummy_metadata();
        metadata.ts_out = ts_out;
        metadata.upgrade(VersionUpgradePolicy::UpgradeToV4);
        Self { decoder, metadata }
    }
}

impl<D> DbnMetadata for FragDecoder<D> {
    fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl<D> DecodeRecordRef<v4::RecordHeader> for FragDecoder<D>
where
    D: DecodeRecordRef<v4::RecordHeader>,
{
    fn decode_record_ref(&mut self) -> dbn::Result<Option<v4::RecordRef<'_>>> {
        self.decoder.decode_record_ref()
    }
}

fn split_encode_impl<D, S>(args: &Args, mut decoder: D, mut splitter: S) -> anyhow::Result<()>
where
    D: DecodeRecordRef<v4::RecordHeader> + DbnMetadata,
    S: Splitter<Encoder>,
{
    let metadata = decoder.metadata().clone();
    let symbol_map = symbol_map(args, &metadata)?;
    while let Some(record) = decoder.decode_record_ref()? {
        if is_framing_rtype(record.header().rtype) {
            check_record_group(args, false)?;
        } else if let Some(encoder) = splitter.sub_encoder(Some(&metadata), &record)? {
            encoder.encode(record, symbol_map.as_ref())?;
        }
    }
    splitter.into_sub_encoders().try_for_each(Encoder::finish)?;
    Ok(())
}

fn symbol_map(args: &Args, metadata: &Metadata) -> dbn::Result<Option<TsSymbolMap>> {
    args.map_symbols
        .then(|| TsSymbolMap::from_metadata(metadata))
        .transpose()
}

fn check_record_group(args: &Args, can_pass_through: bool) -> anyhow::Result<()> {
    if args.map_symbols
        || (!can_pass_through && infer_encoding(args)?.encoding == OutputEncoding::Dbn)
    {
        return Err(anyhow!(
            "Record groups aren't supported yet with --map-symbols or when merging or splitting DBN output"
        ));
    }
    Ok(())
}
