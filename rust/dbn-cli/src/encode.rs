use std::{io, path::Path};

use dbn::{
    decode::{DbnMetadata, DecodeRecordRef},
    encode::{
        json, DbnEncodable, DbnRecordEncoder, DynEncoder, DynWriter, EncodeDbn, EncodeRecord,
        EncodeRecordRef, EncodeRecordTextExt, NoSchemaBehavior, SchemaSplitter, SplitEncoder,
        Splitter, SymbolSplitter, TimeSplitter,
    },
    rtype_dispatch, Compression, Encoding, Metadata, MetadataBuilder, SType, Schema, SymbolIndex,
    TsSymbolMap,
};

use crate::{infer_encoding, output_from_args, Args, InferredEncoding, SplitBy};

pub fn silence_broken_pipe(err: anyhow::Error) -> anyhow::Result<()> {
    // Handle broken pipe as a non-error.
    if let Some(err) = err.downcast_ref::<dbn::Error>() {
        if matches!(err, dbn::Error::Io { source, .. } if source.kind() == std::io::ErrorKind::BrokenPipe)
        {
            return Ok(());
        }
    }
    Err(err)
}

pub fn encode_from_dbn<D>(args: &Args, mut decoder: D) -> anyhow::Result<()>
where
    D: DecodeRecordRef + DbnMetadata,
{
    let writer = output_from_args(args)?;
    let InferredEncoding {
        encoding,
        is_fragment,
        delimiter,
        compression,
    } = infer_encoding(args)?;
    if args.should_output_metadata {
        if encoding != Encoding::Json {
            return Err(anyhow::format_err!(
                "Metadata flag is only valid with JSON encoding"
            ));
        }
        json::Encoder::new(
            writer,
            args.should_pretty_print,
            args.should_pretty_print,
            args.should_pretty_print,
        )
        .encode_metadata(decoder.metadata())?;
    } else if is_fragment {
        encode_fragment(decoder, writer, compression)?;
    } else {
        let mut encoder = DynEncoder::builder(writer, encoding, compression, decoder.metadata())
            .delimiter(delimiter)
            .write_header(args.write_header)
            .all_pretty(args.should_pretty_print)
            .with_symbol(args.map_symbols)
            .build()?;
        if args.map_symbols {
            let symbol_map = decoder.metadata().symbol_map()?;
            let ts_out = decoder.metadata().ts_out;
            while let Some(rec) = decoder.decode_record_ref()? {
                let sym = symbol_map.get_for_rec(&rec).map(String::as_str);
                // SAFETY: `ts_out` is accurate because it's sourced from the metadata
                unsafe {
                    encoder.encode_ref_ts_out_with_sym(rec, ts_out, sym)?;
                }
            }
        } else {
            encoder.encode_decoded(decoder)?;
        }
    }
    Ok(())
}

pub fn split_encode_from_dbn<D>(
    args: &Args,
    split_by: SplitBy,
    output_pattern: &str,
    decoder: D,
) -> anyhow::Result<()>
where
    D: DecodeRecordRef + DbnMetadata,
{
    let InferredEncoding {
        encoding,
        compression,
        delimiter,
        is_fragment: is_output_fragment,
    } = infer_encoding(args)?;
    let open_output = |path: &str| {
        crate::output(Some(Path::new(path)), args.force)
            .map_err(|e| dbn::Error::io(io::Error::other(e), format!("opening output file {path}")))
    };
    if is_output_fragment {
        let build_encoder = |path: &str, _metadata: Option<Metadata>| -> dbn::Result<_> {
            Ok(DbnRecordEncoder::new(DynWriter::new(
                open_output(path)?,
                compression,
            )?))
        };
        split_by_encode_fragment(decoder, split_by, output_pattern, build_encoder)
    } else {
        let build_encoder = |path: &str, metadata: Option<Metadata>| -> dbn::Result<_> {
            DynEncoder::builder(
                open_output(path)?,
                encoding,
                compression,
                &metadata.unwrap(),
            )
            .delimiter(delimiter)
            .write_header(args.write_header)
            .all_pretty(args.should_pretty_print)
            .with_symbol(args.map_symbols)
            .build()
        };
        split_by_encode(
            decoder,
            split_by,
            output_pattern,
            build_encoder,
            args.map_symbols,
        )
    }
}

fn split_by_encode<D, E, F>(
    decoder: D,
    split_by: SplitBy,
    output_pattern: &str,
    build_encoder: F,
    map_symbols: bool,
) -> anyhow::Result<()>
where
    D: DecodeRecordRef + DbnMetadata,
    E: EncodeRecordTextExt,
    F: Fn(&str, Option<Metadata>) -> dbn::Result<E>,
{
    let symbol_map = decoder.metadata().symbol_map()?;
    match split_by {
        SplitBy::Symbol => {
            // TODO: detect live data and split on live symbol mapping msgs
            let splitter = SymbolSplitter::new(
                |symbol: &str, metadata| {
                    build_encoder(&output_pattern.replace("{symbol}", symbol), metadata)
                },
                symbol_map.clone(),
            );
            split_encode_impl(decoder, map_symbols, splitter, Some(symbol_map))
        }
        SplitBy::Schema => {
            let splitter = SchemaSplitter::new(
                |schema: Schema, metadata| {
                    build_encoder(
                        &output_pattern.replace("{schema}", schema.as_str()),
                        metadata,
                    )
                },
                // TODO: support other behaviors
                NoSchemaBehavior::default(),
            );
            split_encode_impl(decoder, map_symbols, splitter, Some(symbol_map))
        }
        SplitBy::Day | SplitBy::Week | SplitBy::Month => {
            let splitter = TimeSplitter::new(
                |date: time::Date, metadata| {
                    build_encoder(
                        &output_pattern.replace("{date}", &date.to_string()),
                        metadata,
                    )
                },
                split_by.duration().unwrap(),
            );
            split_encode_impl(decoder, map_symbols, splitter, Some(symbol_map))
        }
    }
}

fn split_by_encode_fragment<D, E, F>(
    decoder: D,
    split_by: SplitBy,
    output_pattern: &str,
    build_encoder: F,
) -> anyhow::Result<()>
where
    D: DecodeRecordRef + DbnMetadata,
    E: EncodeRecord + EncodeRecordRef,
    F: Fn(&str, Option<Metadata>) -> dbn::Result<E>,
{
    match split_by {
        SplitBy::Symbol => {
            let symbol_map = decoder.metadata().symbol_map()?;
            let splitter = SymbolSplitter::new(
                |symbol: &str, metadata| {
                    build_encoder(&output_pattern.replace("{symbol}", symbol), metadata)
                },
                symbol_map,
            );
            split_encode_fragment_impl(decoder, splitter)
        }
        SplitBy::Schema => {
            let splitter = SchemaSplitter::new(
                |schema: Schema, metadata| {
                    build_encoder(
                        &output_pattern.replace("{schema}", schema.as_str()),
                        metadata,
                    )
                },
                // TODO: support other behaviors
                NoSchemaBehavior::default(),
            );
            split_encode_fragment_impl(decoder, splitter)
        }
        SplitBy::Day | SplitBy::Week | SplitBy::Month => {
            let splitter = TimeSplitter::new(
                |date: time::Date, metadata| {
                    build_encoder(
                        &output_pattern.replace("{date}", &date.to_string()),
                        metadata,
                    )
                },
                split_by.duration().unwrap(),
            );
            split_encode_fragment_impl(decoder, splitter)
        }
    }
}

fn split_encode_impl<D, S, E>(
    mut decoder: D,
    map_symbols: bool,
    splitter: S,
    symbol_map: Option<TsSymbolMap>,
) -> anyhow::Result<()>
where
    D: DecodeRecordRef + DbnMetadata,
    S: Splitter<E>,
    E: EncodeRecordTextExt,
{
    let mut encoder = SplitEncoder::with_metadata(splitter, decoder.metadata().clone());
    if map_symbols {
        let symbol_map = if let Some(symbol_map) = symbol_map {
            symbol_map
        } else {
            decoder.metadata().symbol_map()?
        };
        let ts_out = decoder.metadata().ts_out;
        while let Some(rec) = decoder.decode_record_ref()? {
            let sym = symbol_map.get_for_rec(&rec).map(String::as_str);
            // SAFETY: `ts_out` is accurate because it's sourced from the metadata
            unsafe {
                encoder.encode_ref_ts_out_with_sym(rec, ts_out, sym)?;
            }
        }
    } else {
        encoder.encode_decoded(decoder)?;
    }
    Ok(())
}

fn split_encode_fragment_impl<D, S, E>(mut decoder: D, splitter: S) -> anyhow::Result<()>
where
    D: DecodeRecordRef,
    S: Splitter<E>,
    E: EncodeRecord + EncodeRecordRef,
{
    let mut encoder = SplitEncoder::records_only(splitter);
    while let Some(rec) = decoder.decode_record_ref()? {
        encoder.encode_record_ref(rec)?;
    }
    encoder.flush()?;
    Ok(())
}

pub fn encode_from_frag<D>(args: &Args, mut decoder: D) -> anyhow::Result<()>
where
    D: DecodeRecordRef,
{
    let writer = output_from_args(args)?;
    let InferredEncoding {
        encoding,
        compression,
        delimiter,
        is_fragment,
    } = infer_encoding(args)?;
    if is_fragment {
        encode_fragment(decoder, writer, compression)?;
        return Ok(());
    }
    assert!(!args.should_output_metadata);

    let mut encoder = DynEncoder::builder(
        writer,
        encoding,
        compression,
        // dummy metadata won't be encoded
        &dummy_metadata(),
    )
    .delimiter(delimiter)
    // Can't write header until we know the record type
    .write_header(false)
    .all_pretty(args.should_pretty_print)
    .build()?;
    let mut has_written_header = (encoding != Encoding::Csv) || !args.write_header;
    fn write_header<T: DbnEncodable>(
        _record: &T,
        encoder: &mut DynEncoder<Box<dyn io::Write>>,
    ) -> dbn::Result<()> {
        encoder.encode_header::<T>(false)
    }
    while let Some(record) = decoder.decode_record_ref()? {
        if !has_written_header {
            rtype_dispatch!(record, write_header(&mut encoder))??;
            has_written_header = true;
        }
        encoder.encode_record_ref(record)?;
    }
    Ok(())
}

fn dummy_metadata() -> Metadata {
    MetadataBuilder::new()
        .dataset(String::new())
        .schema(None)
        .start(0)
        .stype_in(None)
        .stype_out(SType::InstrumentId)
        .build()
}

fn encode_fragment<D: DecodeRecordRef>(
    mut decoder: D,
    writer: Box<dyn io::Write>,
    compression: Compression,
) -> dbn::Result<()> {
    let mut encoder = DbnRecordEncoder::new(DynWriter::new(writer, compression)?);
    while let Some(record) = decoder.decode_record_ref()? {
        encoder.encode_record_ref(record)?;
    }
    Ok(())
}

/// Split encode from a fragment input (no metadata).
///
/// Only supports time-based and schema-based splitting. Symbol splitting requires
/// a symbol map which is not available in fragment inputs.
pub fn split_encode_from_frag<D>(
    args: &Args,
    split_by: SplitBy,
    output_pattern: &str,
    decoder: D,
) -> anyhow::Result<()>
where
    D: DecodeRecordRef,
{
    if matches!(split_by, SplitBy::Symbol) {
        return Err(anyhow::anyhow!(
            "Cannot split by symbol when input is a fragment: no symbol map available"
        ));
    }
    let InferredEncoding {
        encoding,
        compression,
        delimiter,
        is_fragment,
    } = infer_encoding(args)?;
    let open_output = |path: &str| {
        crate::output(Some(Path::new(path)), args.force)
            .map_err(|e| dbn::Error::io(io::Error::other(e), format!("opening output file {path}")))
    };
    if is_fragment {
        let build_encoder = |path: &str| -> dbn::Result<_> {
            Ok(DbnRecordEncoder::new(DynWriter::new(
                open_output(path)?,
                compression,
            )?))
        };
        match split_by {
            SplitBy::Symbol => unreachable!("handled above"),
            SplitBy::Schema => {
                let splitter = SchemaSplitter::new(
                    |schema: Schema, _metadata| {
                        build_encoder(&output_pattern.replace("{schema}", schema.as_str()))
                    },
                    NoSchemaBehavior::default(),
                );
                split_encode_fragment_impl(decoder, splitter)
            }
            SplitBy::Day | SplitBy::Week | SplitBy::Month => {
                let splitter = TimeSplitter::new(
                    |date: time::Date, _metadata| {
                        build_encoder(&output_pattern.replace("{date}", &date.to_string()))
                    },
                    split_by.duration().unwrap(),
                );
                split_encode_fragment_impl(decoder, splitter)
            }
        }
    } else {
        let metadata = dummy_metadata();
        let build_encoder = |path: &str| -> dbn::Result<_> {
            DynEncoder::builder(open_output(path)?, encoding, compression, &metadata)
                .delimiter(delimiter)
                .write_header(args.write_header)
                .all_pretty(args.should_pretty_print)
                .build()
        };
        match split_by {
            SplitBy::Symbol => unreachable!("handled above"),
            SplitBy::Schema => {
                let splitter = SchemaSplitter::new(
                    |schema: Schema, _metadata| {
                        build_encoder(&output_pattern.replace("{schema}", schema.as_str()))
                    },
                    NoSchemaBehavior::default(),
                );
                split_encode_fragment_impl(decoder, splitter)
            }
            SplitBy::Day | SplitBy::Week | SplitBy::Month => {
                let splitter = TimeSplitter::new(
                    |date: time::Date, _metadata| {
                        build_encoder(&output_pattern.replace("{date}", &date.to_string()))
                    },
                    split_by.duration().unwrap(),
                );
                split_encode_fragment_impl(decoder, splitter)
            }
        }
    }
}
