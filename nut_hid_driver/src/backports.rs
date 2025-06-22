/// Backported from feature str_from_utf16_endian
pub fn from_utf16le_lossy(v: &[u8]) -> String {
    match (cfg!(target_endian = "little"), unsafe {
        v.align_to::<u16>()
    }) {
        (true, ([], v, [])) => String::from_utf16_lossy(v),
        (true, ([], v, [_remainder])) => String::from_utf16_lossy(v) + "\u{FFFD}",
        _ => {
            panic!("Only little ending supported for now");
        }
    }
}
