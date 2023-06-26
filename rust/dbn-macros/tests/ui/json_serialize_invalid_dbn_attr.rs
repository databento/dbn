use dbn_macros::JsonSerialize;

#[derive(JsonSerialize)]
#[repr(C)]
struct Record {
    #[dbn(unknown)]
    pub a: u8,
}

fn main() {}
