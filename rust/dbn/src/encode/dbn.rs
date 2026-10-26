//! Encoding DBN records into DBN, Zstandard-compressed or not.

use std::{
    io::{IoSlice, Write},
    mem::{transmute, MaybeUninit},
};

use crate::metadata::MetadataLengths;

/// Maximum number of `IoSlice`s to use in a single vectored write call.
const MAX_SLICES: usize = 128;

/// Increment added to a record's header `length` field (in `LENGTH_MULTIPLIER` units)
/// when appending a `ts_out` timestamp.
const TS_OUT_LEN: u8 = (std::mem::size_of::<u64>() / crate::RecordHeader::LENGTH_MULTIPLIER) as u8;

fn write_records<W, H>(writer: &mut W, records: &[crate::RecordRef<'_, H>]) -> std::io::Result<()>
where
    W: Write,
    H: crate::record::RecordHeaderKind,
{
    let mut slices = [const { MaybeUninit::uninit() }; MAX_SLICES];
    for chunk in records.chunks(MAX_SLICES) {
        for (elem, rec) in slices.iter_mut().zip(chunk.iter()) {
            elem.write(IoSlice::from(*rec));
        }
        let slices =
            // SAFETY: every element up to `chunk.len()` was just initialized
            unsafe { transmute::<&mut [MaybeUninit<IoSlice<'_>>], &mut [IoSlice<'_>]>(&mut slices[..chunk.len()]) };
        crate::encode::io_utils::write_all_vectored(writer, slices)?;
    }
    Ok(())
}

#[cfg(feature = "async")]
async fn write_records_async<W, H>(
    writer: &mut W,
    records: &[crate::RecordRef<'_, H>],
) -> std::io::Result<()>
where
    W: tokio::io::AsyncWriteExt + Unpin,
    H: crate::record::RecordHeaderKind,
{
    let mut slices = [const { MaybeUninit::uninit() }; MAX_SLICES];
    for chunk in records.chunks(MAX_SLICES) {
        for (elem, rec) in slices.iter_mut().zip(chunk.iter()) {
            elem.write(IoSlice::from(*rec));
        }
        let slices =
            // SAFETY: every element up to `chunk.len()` was just initialized
            unsafe { transmute::<&mut [MaybeUninit<IoSlice<'_>>], &mut [IoSlice<'_>]>(&mut slices[..chunk.len()]) };
        crate::encode::io_utils::write_all_vectored_async(writer, slices).await?;
    }
    Ok(())
}

fn check_pre_v4(metadata: &crate::Metadata) -> crate::Result<()> {
    if metadata.version >= crate::v4::DBN_VERSION {
        return Err(crate::Error::encode(format!(
            "this encoder writes DBN versions 1 to 3, not {}; use `encode::dbn::v4::Encoder`",
            metadata.version
        )));
    }
    Ok(())
}

fn metadata_lengths(version: u8) -> crate::Result<&'static MetadataLengths> {
    MetadataLengths::for_version(version).ok_or_else(|| {
        crate::Error::encode(format!(
            "can't encode Metadata with version {version} which is greater than the maximum \
             supported version {}",
            crate::v4::DBN_VERSION
        ))
    })
}

fn layout_of<'a>(
    lengths: &MetadataLengths,
    metadata: &'a crate::Metadata,
) -> crate::Result<Option<&'a crate::layout::StreamLayout>> {
    match (lengths.has_layout, metadata.layout.as_ref()) {
        (true, Some(layout)) => Ok(Some(layout)),
        (true, None) => Err(crate::Error::encode(
            "a DBN version 4 stream must carry a stream layout",
        )),
        (false, Some(_)) => Err(crate::Error::encode(format!(
            "DBN version {} has nowhere to put a stream layout",
            metadata.version
        ))),
        (false, None) => Ok(None),
    }
}

fn encode_layout(lengths: &MetadataLengths, metadata: &crate::Metadata) -> crate::Result<Vec<u8>> {
    let mut buf = Vec::new();
    if let Some(layout) = layout_of(lengths, metadata)? {
        crate::layout::validate_for_metadata(
            layout,
            metadata.symbol_cstr_len as u16,
            metadata.ts_out,
        )?;
        crate::layout::wire::encode(&mut buf, layout)?;
    }
    Ok(buf)
}

fn check_symbology(lengths: &MetadataLengths, metadata: &crate::Metadata) -> crate::Result<()> {
    if lengths.has_symbol_intervals && !metadata.mappings.is_empty() {
        return Err(crate::Error::encode(format!(
            "DBN version {} carries symbology in `intervals`, not `mappings`",
            metadata.version
        )));
    }
    if !lengths.has_symbol_intervals && !metadata.intervals.is_empty() {
        return Err(crate::Error::encode(format!(
            "DBN version {} carries symbology in `mappings`, not `intervals`",
            metadata.version
        )));
    }
    Ok(())
}

fn symbology_length(lengths: &MetadataLengths, metadata: &crate::Metadata) -> usize {
    const COUNT_LEN: usize = std::mem::size_of::<u32>();
    if lengths.has_symbol_intervals {
        let ids_len = std::mem::size_of::<u64>() * 3;
        let strs_len = |strs: &[String]| COUNT_LEN + crate::layout::wire::strs_len(strs);
        let intervals_len = COUNT_LEN
            + metadata
                .intervals
                .iter()
                .map(|i| {
                    ids_len
                        + crate::layout::wire::str_len(&i.stype_in_symbol)
                        + crate::layout::wire::str_len(&i.stype_out_symbol)
                })
                .sum::<usize>();
        return strs_len(&metadata.symbols)
            + strs_len(&metadata.partial)
            + strs_len(&metadata.not_found)
            + intervals_len;
    }
    let symbol_len = metadata.symbol_cstr_len;
    let cstrs_len = |cstrs: &[String]| COUNT_LEN + cstrs.len() * symbol_len;
    let interval_len = std::mem::size_of::<u32>() * 2 + symbol_len;
    let mappings_len = COUNT_LEN
        + metadata
            .mappings
            .iter()
            .map(|m| symbol_len + COUNT_LEN + m.intervals.len() * interval_len)
            .sum::<usize>();
    cstrs_len(&metadata.symbols)
        + cstrs_len(&metadata.partial)
        + cstrs_len(&metadata.not_found)
        + mappings_len
}

fn calc_length(
    lengths: &MetadataLengths,
    metadata: &crate::Metadata,
    layout_len: usize,
) -> (u32, u32) {
    let length_prefixes =
        std::mem::size_of::<u32>() * if lengths.has_symbol_intervals { 2 } else { 1 };
    let needed_len = (crate::METADATA_FIXED_LEN
        + length_prefixes
        + layout_len
        + symbology_length(lengths, metadata)) as u32;
    let rem = needed_len % 8;
    if !lengths.should_pad_to_8 || rem == 0 {
        (needed_len, 0)
    } else {
        let end_padding = 8 - rem;
        (needed_len + end_padding, end_padding)
    }
}

mod sync;
pub mod v4;
pub use sync::{Encoder, MetadataEncoder, RecordEncoder};

#[cfg(feature = "async")]
mod r#async;
#[cfg(feature = "async")]
pub use r#async::{
    Encoder as AsyncEncoder, MetadataEncoder as AsyncMetadataEncoder,
    RecordEncoder as AsyncRecordEncoder,
};
