pub fn insensitive_contains(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().all(|needle| contains(haystack, needle))
}

fn contains(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    
    if haystack.is_ascii() && needle.is_ascii() {
        return haystack.as_bytes()
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle.as_bytes()));
    }
    
    haystack.to_lowercase().contains(&needle.to_lowercase())
}
