use dbn_macros::JsonSerialize;

#[derive(JsonSerialize)]
#[repr(C)]
struct Record {
    #[dbn(fixed_price, unix_nanos)]
    pub a: u8,
}

fn main() {}
