use dbn::v4::RecordHeader;
use dbn_macros::RecordLayoutDesc;

#[repr(C)]
#[derive(RecordLayoutDesc)]
struct Record {
    hd: RecordHeader,
    size: i32,
}

fn main() {}
