#![allow(unexpected_cfgs)]

use dbn::{
    macros::{dbn_record, DbnAttr},
    v4::{rtype, RecordHeader},
    Side,
};

#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq)]
#[dbn_record(rtype::INSTRUMENT_DEF)]
struct Record {
    hd: RecordHeader,
    security_update_action: Side,
}

fn main() {}
