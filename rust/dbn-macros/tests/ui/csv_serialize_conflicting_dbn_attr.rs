use dbn_macros::CsvSerialize;

#[derive(CsvSerialize)]
#[repr(C)]
struct Record {
    #[dbn(fixed_price, unix_nanos)]
    pub a: u8,
}

fn main() {}
