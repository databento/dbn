use dbn_macros::dbn_record;

#[repr(C)]
#[dbn_record]
struct Record {
    pub a: u8,
}

fn main() {}
