pub fn win_string(base: &str) -> Vec<u16> {
    base.encode_utf16()
        .chain(Some(0))
        .collect::<Vec<u16>>()
}
