//! Framing records: pseudo-records that structure a DBN version 4 stream rather than
//! carry market data.
//!
//! A framing record begins with `length` and `rtype` like a record header, followed by a
//! `u32` `trailer_length`: the number of bytes after `24 + length` that also belong to
//! the record. A reader can skip any framing record, including ones it doesn't
//! recognize, using these lengths.

use crate::decode::FromLittleEndianSlice;

/// Returns `true` if `rtype` is in the range reserved for framing records.
pub const fn is_framing_rtype(rtype: u16) -> bool {
    matches!(rtype, 0x00F0..=0x00FF)
}

/// The fixed part of the framing record that begins each record group.
/// A trailer of `trailer_length` bytes follows it, holding an optional stream layout
/// and then the group's symbol intervals.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordGroupHeaderMsg {
    /// The byte length of the fields after the first 24 bytes. Always 8.
    pub length: u16,
    /// The record type. Always [`Self::RTYPE`].
    pub rtype: u16,
    /// The byte length of the trailer following the fixed 32 bytes.
    pub trailer_length: u32,
    /// The fingerprint of the trailer's stream layout, or 0 when there is none.
    pub stream_layout_hash: u64,
    /// The index timestamp of the group's first record expressed as the number of
    /// nanoseconds since the UNIX epoch.
    pub start_ts: u64,
    /// The byte length of the stream layout in the trailer. 0 means the stream
    /// layout is unchanged.
    pub stream_layout_length: u32,
    /// The number of symbol intervals in the trailer after the stream layout.
    pub interval_count: u32,
}

/// A parsed [`RecordGroupHeaderMsg`] with its trailer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordGroupHeader<'a> {
    /// The fixed part of the record.
    pub msg: RecordGroupHeaderMsg,
    /// The encoded symbol intervals followed by padding to a multiple of 8 bytes.
    pub symbol_map: &'a [u8],
}

impl RecordGroupHeaderMsg {
    /// The framing record type of a record group header.
    pub const RTYPE: u16 = 0x00FE;
}

impl<'a> RecordGroupHeader<'a> {
    /// Parses a whole record group header frame, including its trailer.
    ///
    /// # Errors
    /// This function returns an error if `frame` isn't a complete record group header,
    /// its lengths disagree, or it changes the stream layout, which isn't supported.
    pub fn parse(frame: &'a [u8]) -> crate::Result<Self> {
        const FIXED_LEN: usize = std::mem::size_of::<RecordGroupHeaderMsg>();

        if frame.len() < FIXED_LEN {
            return Err(crate::Error::decode(format!(
                "record group header of {} bytes is shorter than {FIXED_LEN}",
                frame.len()
            )));
        }
        let msg = RecordGroupHeaderMsg {
            length: u16::from_le_slice(frame),
            rtype: u16::from_le_slice(&frame[2..]),
            trailer_length: u32::from_le_slice(&frame[4..]),
            stream_layout_hash: u64::from_le_slice(&frame[8..]),
            start_ts: u64::from_le_slice(&frame[16..]),
            stream_layout_length: u32::from_le_slice(&frame[24..]),
            interval_count: u32::from_le_slice(&frame[28..]),
        };
        if msg.rtype != RecordGroupHeaderMsg::RTYPE {
            return Err(crate::Error::decode(format!(
                "expected record group header rtype {:#06x}, found {:#06x}",
                RecordGroupHeaderMsg::RTYPE,
                msg.rtype
            )));
        }
        if usize::from(msg.length) != FIXED_LEN - std::mem::size_of::<crate::v4::RecordHeader>() {
            return Err(crate::Error::decode(format!(
                "invalid record group header length {}",
                msg.length
            )));
        }
        if msg.stream_layout_length > msg.trailer_length {
            return Err(crate::Error::decode(format!(
                "record group header stream layout length {} exceeds trailer length {}",
                msg.stream_layout_length, msg.trailer_length
            )));
        }
        if msg.stream_layout_length != 0 {
            return Err(crate::Error::decode(
                "changing the stream layout in a record group header isn't supported",
            ));
        }
        let trailer = frame[FIXED_LEN..]
            .get(..msg.trailer_length as usize)
            .ok_or_else(|| {
                crate::Error::decode(format!(
                    "record group header trailer length {} exceeds the {} bytes after the header",
                    msg.trailer_length,
                    frame.len() - FIXED_LEN
                ))
            })?;
        Ok(Self {
            msg,
            symbol_map: &trailer[msg.stream_layout_length as usize..],
        })
    }
}

/// The common fields that begin every framing record.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FramingHeader {
    pub length: u16,
    pub rtype: u16,
    pub trailer_length: u32,
}

impl FramingHeader {
    pub fn from_bytes(bytes: &[u8; 8]) -> Self {
        Self {
            length: u16::from_le_slice(bytes),
            rtype: u16::from_le_slice(&bytes[2..]),
            trailer_length: u32::from_le_slice(&bytes[4..]),
        }
    }

    pub fn frame_len(&self) -> crate::Result<usize> {
        if !self.trailer_length.is_multiple_of(8) {
            return Err(crate::Error::decode(format!(
                "invalid framing record with trailer length {} which is not a multiple of 8",
                self.trailer_length
            )));
        }
        Ok(std::mem::size_of::<crate::v4::RecordHeader>()
            + usize::from(self.length)
            + self.trailer_length as usize)
    }
}

#[cfg(test)]
mod tests {
    use rstest::*;

    use super::*;

    const START_TS: u64 = 1_697_240_529_000_000_000;

    fn frame(msg: &RecordGroupHeaderMsg, trailer: &[u8]) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.extend_from_slice(&msg.length.to_le_bytes());
        frame.extend_from_slice(&msg.rtype.to_le_bytes());
        frame.extend_from_slice(&msg.trailer_length.to_le_bytes());
        frame.extend_from_slice(&msg.stream_layout_hash.to_le_bytes());
        frame.extend_from_slice(&msg.start_ts.to_le_bytes());
        frame.extend_from_slice(&msg.stream_layout_length.to_le_bytes());
        frame.extend_from_slice(&msg.interval_count.to_le_bytes());
        frame.extend_from_slice(trailer);
        frame
    }

    fn msg(trailer_length: u32) -> RecordGroupHeaderMsg {
        RecordGroupHeaderMsg {
            length: 8,
            rtype: RecordGroupHeaderMsg::RTYPE,
            trailer_length,
            stream_layout_hash: 0,
            start_ts: START_TS,
            stream_layout_length: 0,
            interval_count: 1,
        }
    }

    #[test]
    fn test_record_group_header_msg_is_32_bytes() {
        assert_eq!(std::mem::size_of::<RecordGroupHeaderMsg>(), 32);
        assert!(is_framing_rtype(RecordGroupHeaderMsg::RTYPE));
    }

    #[rstest]
    fn test_parse(#[values(0, 16)] trailer_length: u32) {
        let symbol_map: Vec<u8> = (0..trailer_length).map(|i| i as u8).collect();
        let msg = msg(trailer_length);
        let frame = frame(&msg, &symbol_map);
        assert_eq!(
            RecordGroupHeader::parse(&frame).unwrap(),
            RecordGroupHeader {
                msg,
                symbol_map: &symbol_map
            }
        );
    }

    #[rstest]
    #[case::wrong_rtype(RecordGroupHeaderMsg { rtype: 0xF5, ..msg(16) })]
    #[case::wrong_length(RecordGroupHeaderMsg { length: 16, ..msg(16) })]
    #[case::stream_layout_past_trailer(RecordGroupHeaderMsg { stream_layout_length: 24, ..msg(16) })]
    #[case::stream_layout(RecordGroupHeaderMsg {
        stream_layout_hash: 1,
        stream_layout_length: 8,
        ..msg(16)
    })]
    #[case::trailer_past_frame(msg(24))]
    fn test_parse_rejects(#[case] msg: RecordGroupHeaderMsg) {
        let frame = frame(&msg, &[0; 16]);
        assert!(RecordGroupHeader::parse(&frame).is_err());
    }

    #[test]
    fn test_parse_rejects_short_frame() {
        let frame = frame(&msg(0), &[]);
        assert!(RecordGroupHeader::parse(&frame[..31]).is_err());
    }
}
