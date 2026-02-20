pub fn subslice_index(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let needle_len = needle.len();
    let haystack_len = haystack.len();
    
    if needle_len == 0 || needle_len > haystack_len {
        return None;
    }
    
    let needle_f = needle[0];
    let needle_flo = needle_f.to_ascii_lowercase();
    let needle_fup = needle_f.to_ascii_uppercase();
    
    // only consider sections of the haystack where the needle can fit
    for index in 0..=(haystack_len - needle_len) {
        
        let current = haystack[index];
        
        // if the first character of the needle matches, perform full check
        if current == needle_flo || current == needle_fup {
            if haystack[index..][..needle_len].eq_ignore_ascii_case(needle) {
                return Some(index);
            }
        }
        
    }
    
    None
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn simple() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"holder";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert_eq!(output, Some(5));
    }
    
    #[test]
    fn case_mismatch() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"HOLDER";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert_eq!(output, Some(5));
    }
    
    #[test]
    fn content_match() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"placeholder";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert_eq!(output, Some(0));
    }
    
    #[test]
    fn emoji() {
        // setup
        
        let haystack = "pla🔌ceholder🔌";
        let needle = b"HOLDER";
        
        // operation
        
        let output = subslice_index(haystack.as_bytes(), needle);
        
        // control
        
        assert_eq!(output, Some(9));
    }
    
    #[test]
    fn not_contained() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"HODLER";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn empty() {
        // setup
        
        let haystack = b"";
        let needle = b"";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn haystack_empty() {
        // setup
        
        let haystack = b"";
        let needle = b"HOLDER";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn needle_empty() {
        // setup
        
        let haystack = b"placeholder";
        let needle = b"";
        
        // operation
        
        let output = subslice_index(haystack, needle);
        
        // control
        
        assert!(output.is_none());
    }
    
}
