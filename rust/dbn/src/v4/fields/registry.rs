//! The DBN v4 global field-ID registry.
//!
//! Generated from the `[fields]` table in `defs/dbn.toml`. Do not edit by hand;
//! change the TOML and rerun codegen. Field IDs are permanent and assigned once.

use super::Field;

/// Field ID for `length`.
pub const LENGTH: Field<u16> = Field::new(0x01);
/// Field ID for `rtype`.
pub const RTYPE: Field<u16> = Field::new(0x02);
/// Field ID for `publisher_id`.
pub const PUBLISHER_ID: Field<u16> = Field::new(0x03);
/// Field ID for `instrument_id`.
pub const INSTRUMENT_ID: Field<u64> = Field::new(0x04);
/// Field ID for `ts_event`.
pub const TS_EVENT: Field<crate::v4::types::TimestampNs> = Field::new(0x05);
/// Field ID for `price`.
pub const PRICE: Field<crate::v4::types::Decimal> = Field::new(0x10);
/// Field ID for `size`.
pub const SIZE: Field<u32> = Field::new(0x11);
/// Field ID for `action`.
pub const ACTION: Field<crate::v4::types::OpenEnum<crate::Action>> = Field::new(0x12);
/// Field ID for `side`.
pub const SIDE: Field<crate::v4::types::OpenEnum<crate::Side>> = Field::new(0x13);
/// Field ID for `flags`.
pub const FLAGS: Field<crate::FlagSet> = Field::new(0x14);
/// Field ID for `depth`.
pub const DEPTH: Field<u8> = Field::new(0x15);
/// Field ID for `ts_recv`.
pub const TS_RECV: Field<crate::v4::types::TimestampNs> = Field::new(0x16);
/// Field ID for `ts_in_delta`.
pub const TS_IN_DELTA: Field<i32> = Field::new(0x17);
/// Field ID for `sequence`.
pub const SEQUENCE: Field<u32> = Field::new(0x18);
/// Field ID for `order_id`.
pub const ORDER_ID: Field<u64> = Field::new(0x19);
/// Field ID for `channel_id`.
pub const CHANNEL_ID: Field<u8> = Field::new(0x1B);
/// Field ID for `hd`, the record header.
pub const HD: u16 = 0x1000;
