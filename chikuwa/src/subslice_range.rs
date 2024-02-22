use std::ops::Range;

pub fn subslice_range(content: &[u8], left: &[u8], right: &[u8]) -> Option<Range<usize>> {
    // windows method panics if given a zero as length
    if left.is_empty() || right.is_empty() {
        return None;
    }
    
    let start = content.windows(left.len())
        .position(|window| window.eq_ignore_ascii_case(left))
        .and_then(|index| index.checked_add(left.len()))?;
    
    let end = content[start..].windows(right.len())
        .position(|window| window.eq_ignore_ascii_case(right))
        .and_then(|index| index.checked_add(start))?;
    
    Some(start..end)
}
