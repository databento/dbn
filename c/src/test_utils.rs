use crate::metadata::StrRef;

pub(crate) unsafe fn str_ref_to_string(s: StrRef) -> String {
    let bytes = std::slice::from_raw_parts(s.data as *const u8, s.len);
    String::from_utf8(bytes.to_vec()).unwrap()
}
