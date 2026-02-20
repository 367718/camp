pub fn split_once<'c>(content: &'c [u8], separator: &[u8]) -> (&'c [u8], &'c [u8]) {
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

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn single() {
        // setup
        
        let content = b"placeholder";
        let separator = b"ho";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"place".as_slice(), b"lder".as_slice()));
    }
    
    #[test]
    fn single_first() {
        // setup
        
        let content = b"placeholder";
        let separator = b"pl";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"".as_slice(), b"aceholder".as_slice()));
    }
    
    #[test]
    fn single_last() {
        // setup
        
        let content = b"placeholder";
        let separator = b"der";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"placehol".as_slice(), b"".as_slice()));
    }
    
    #[test]
    fn multiple() {
        // setup
        
        let content = b"placeholder";
        let separator = b"l";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"p".as_slice(), b"aceholder".as_slice()));
    }
    
    #[test]
    fn empty_content() {
        // setup
        
        let content = b"";
        let separator = b"ho";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"".as_slice(), b"".as_slice()));
    }
    
    #[test]
    fn empty_separator() {
        // setup
        
        let content = b"placeholder";
        let separator = b"";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"placeholder".as_slice(), b"".as_slice()));
    }
    
    #[test]
    fn not_present() {
        // setup
        
        let content = b"placeholder";
        let separator = b"test";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"placeholder".as_slice(), b"".as_slice()));
    }
    
    #[test]
    fn separator_only() {
        // setup
        
        let content = b"test";
        let separator = b"test";
        
        // operation
        
        let output = split_once(content, separator);
        
        // control
        
        assert_eq!(output, (b"".as_slice(), b"".as_slice()));
    }
    
    #[test]
    fn emoji() {
        // setup
        
        let content = "placeh🔌older";
        let separator = "h🔌o";
        
        // operation
        
        let output = split_once(content.as_bytes(), separator.as_bytes());
        
        // control
        
        assert_eq!(output, (b"place".as_slice(), b"lder".as_slice()));
    }
    
}
