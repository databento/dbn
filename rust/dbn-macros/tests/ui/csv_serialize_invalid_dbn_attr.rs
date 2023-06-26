use dbn_macros::CsvSerialize;

#[derive(CsvSerialize)]
#[repr(C)]
struct Record {
    #[dbn(unknown)]
    pub a: u8,
}

fn main() {}
