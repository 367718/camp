pub fn first_number(value: &[u8]) -> Option<u64> {
    let mut bytes = value.iter();
    
    let first_digit = bytes.find(|byte| byte.is_ascii_digit())?;
    
    let mut result = u64::from(first_digit - b'0');
    
    for byte in bytes {
        
        if ! byte.is_ascii_digit() {
            break;
        }
        
        let current_digit = u64::from(byte - b'0');
        
        result = result.checked_mul(10)?.checked_add(current_digit)?;
        
    }
    
    Some(result)
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn valid() {
        // setup
        
        let value = b"[Example] Placeholder - 17 (720p) [83538700].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(17));
    }
    
    #[test]
    fn zero() {
        // setup
        
        let value = b"[Example] Placeholder - 0 (720p) [83538700].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(0));
    }
    
    #[test]
    fn negative() {
        // setup
        
        let value = b"[Example] Placeholder - -13 (720p) [83538700].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(13));
    }
    
    #[test]
    fn limit() {
        // setup
        
        let value = b"[Example] Placeholder - 18446744073709551615 (720p) [83538700].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(18446744073709551615));
    }
    
    #[test]
    fn too_big() {
        // setup
        
        let value = b"[Example] Placeholder - 18446744073709551616 (720p) [83538700].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, None);
    }
    
    #[test]
    fn at_start() {
        // setup
        
        let value = b"14[Example] Placeholder - (p) [AGDFASZ].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(14));
    }
    
    #[test]
    fn at_end() {
        // setup
        
        let value = b"[Example] Placeholder - (p) [AGDFASZ]17.mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(17));
    }
    
    #[test]
    fn number_only() {
        // setup
        
        let value = b"894";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, Some(894));
    }
    
    #[test]
    fn no_number() {
        // setup
        
        let value = b"[Example] Placeholder - (p) [AGDFASZ].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, None);
    }
    
    #[test]
    fn empty() {
        // setup
        
        let value = b"";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, None);
    }
    
}
