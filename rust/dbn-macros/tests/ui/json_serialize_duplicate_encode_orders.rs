use dbn_macros::JsonSerialize;

#[derive(JsonSerialize)]
#[repr(C)]
struct Record {
    #[dbn(encode_order(1))]
    pub a: u8,
    pub b: u8,
    #[dbn(encode_order(1))]
    pub c: u8,
}

fn main() {}
