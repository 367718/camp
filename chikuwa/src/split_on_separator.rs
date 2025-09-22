pub fn split_on_separator<'c>(content: &'c [u8], separator: &[u8]) -> (&'c [u8], &'c [u8]) {
    extract_pair(content, separator)
        .unwrap_or((content, &[]))
}

fn extract_pair<'c>(content: &'c [u8], separator: &[u8]) -> Option<(&'c [u8], &'c [u8])> {
    // 'windows' function panics on 0 length
    if separator.is_empty() {
        return None;
    }
    
    let position = content.windows(separator.len())
        .position(|window| window.eq_ignore_ascii_case(separator))?;
    
    let left = &content[..position];
    let right = &content[position..][separator.len()..];
    
    Some((left, right))
}
