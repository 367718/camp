pub fn win_string(content: &str) -> Vec<u16> {
    content.encode_utf16()
        .chain(Some(0))
        .collect::<Vec<u16>>()
}
