pub fn insensitive_contains(haystack: &[u8], needle: &[u8]) -> bool {
    // 'windows' function panics on 0 length
    if needle.is_empty() {
        return true;
    }
    
    haystack.windows(needle.len())
        .any(|curr| curr.eq_ignore_ascii_case(needle))
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn simple() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"HOLDER";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == true);
    }
    
    #[test]
    fn case_match() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"holder";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == true);
    }
    
    #[test]
    fn content_match() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"placeholder";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == true);
    }
    
    #[test]
    fn emoji() {
        // setup
        
        let haystack = "pla🔌ceholder🔌";
        let needle = b"HOLDER";
        
        // operation
        
        let output = insensitive_contains(haystack.as_bytes(), needle);
        
        // control
        
        assert!(output == true);
    }
    
    #[test]
    fn not_contained() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"HODLER";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == false);
    }
    
    #[test]
    fn empty() {
        // setup
        
        let haystack = b"";
        let needle = b"";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == true);
    }
    
    #[test]
    fn haystack_empty() {
        // setup
        
        let haystack = b"";
        let needle = b"HOLDER";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == false);
    }
    
    #[test]
    fn needle_empty() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"";
        
        // operation
        
        let output = insensitive_contains(haystack, needle);
        
        // control
        
        assert!(output == true);
    }
    
}
