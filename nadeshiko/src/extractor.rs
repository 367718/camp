pub fn get(clean: &str) -> Option<u32> {
    let mut chars = clean.chars();
    
    while let Some(curr) = chars.next() {
        
        if let Some(digit) = curr.to_digit(10) {
            
            let mut number = digit;
            
            while let Some(curr) = chars.next() {
                
                if let Some(digit) = curr.to_digit(10) {
                    number = number.checked_mul(10)?.checked_add(digit)?;
                    continue;
                }
                
                // if next to a digit is a dot and next to the dot is another digit, abort
                if curr == '.' && chars.next().filter(char::is_ascii_digit).is_some() {
                    return None;
                }
                
                break;
                
            }
            
            if number > 0 {
                return Some(number);
            }
            
            break;
            
        }
        
    }
    
    None
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn clean() {
            // setup
            
            let value = "[]  - 10 []";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, Some(10));
        }
        
        #[test]
        fn unclean() {
            // setup
            
            let value = "[Imaginary] Fictional 86 - 10 [720p]";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, Some(86));
        }
        
        #[test]
        fn zero() {
            // setup
            
            let value = "[Non-existent] Made up - 0 [720p]";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn negative() {
            // setup
            
            let value = "[Non-existent] Made up - -17 [720p]";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, Some(17));
        }
        
        #[test]
        fn decimal() {
            // setup
            
            let value = "[Non-existent] Made up - 12.5 [720p]";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn whitespace() {
            // setup
            
            let value = "[]  - 2 4 []";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, Some(2));
        }
        
        #[test]
        fn big() {
            // setup
            
            let value = "[]  - 42949672 []";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, Some(42949672));
        }
        
        #[test]
        fn overflow() {
            // setup
            
            let value = "[]  - 4294967296 []";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn empty() {
            // setup
            
            let value = "";
            
            // operation
            
            let output = get(value);
            
            // control
            
            assert_eq!(output, None);
        }
        
    }
    
}
