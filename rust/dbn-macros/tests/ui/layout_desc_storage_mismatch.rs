#![allow(unexpected_cfgs)]

use dbn::{
    macros::{dbn_record, DbnAttr},
    v4::{rtype, RecordHeader},
};

#[repr(C)]
#[derive(Clone, DbnAttr, PartialEq)]
#[dbn_record(rtype::MBP_0)]
struct Record {
    hd: RecordHeader,
    size: i32,
}

fn main() {}
