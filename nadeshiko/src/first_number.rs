pub fn first_number(value: &[u8]) -> Option<u64> {
    let mut chars = value
        .iter()
        .copied()
        .map(char::from);
    
    let mut episode = chars.find_map(|character| character.to_digit(10).map(u64::from))?;
    
    while let Some(digit) = chars.next().and_then(|character| character.to_digit(10).map(u64::from)) {
        episode = episode.checked_mul(10)?.checked_add(digit)?;
    }
    
    Some(episode)
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
    fn no_number() {
        // setup
        
        let value = b"[Example] Placeholder - (p) [AGDFASZ].mkv";
        
        // operation
        
        let output = first_number(value);
        
        // control
        
        assert_eq!(output, None);
    }
    
}
