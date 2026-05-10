pub fn subslice_index(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    let needle_len = needle.len();
    
    if needle_len == 0 || needle_len > haystack.len() {
        return None;
    }
    
    let first_byte = needle[0];
    let first_lower = first_byte.to_ascii_lowercase();
    let first_upper = first_byte.to_ascii_uppercase();
    
    // avoid re-checking first byte
    let needle_rest = &needle[1..];
    
    // first character might not be alphabetic (e.g. '<' or ' ')
    let non_alphabetic = first_lower == first_upper;
    
    let mut index = 0;
    let mut haystack_rest = haystack;
    
    while haystack_rest.len() >= needle_len {
        
        let position = if non_alphabetic {
            haystack_rest.iter().position(|&byte| byte == first_lower)
        } else {
            haystack_rest.iter().position(|&byte| byte == first_lower || byte == first_upper)
        }?;
        
        index += position;
        haystack_rest = &haystack_rest[position..];
        
        if haystack_rest.len() < needle_len {
            return None;
        }
        
        // first byte matches, compare rest
        if haystack_rest[1..needle_len].eq_ignore_ascii_case(needle_rest) {
            return Some(index);
        }
        
        // skip over matched byte and continue search
        index += 1;
        haystack_rest = &haystack_rest[1..];
        
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
