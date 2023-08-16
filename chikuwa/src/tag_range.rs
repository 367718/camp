use std::ops::Range;

pub fn tag_range(content: &[u8], open: &[u8], close: &[u8]) -> Option<Range<usize>> {
    let start = content.windows(open.len())
        .position(|window| window.eq_ignore_ascii_case(open))
        .and_then(|index| index.checked_add(open.len()))?;
    
    let end = content[start..].windows(close.len())
        .position(|window| window.eq_ignore_ascii_case(close))
        .and_then(|index| index.checked_add(start))?;
    
    Some(start..end)
}