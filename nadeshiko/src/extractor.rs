use std::ops::Range;

pub fn get(value: &str, pieces: &[&str]) -> Option<i64> {
    // remove pieces from supplied value
    
    let indexes: Vec<Range<usize>> = pieces.iter()
        .filter_map(|piece| value.match_indices(piece).next())
        .map(|(start, piece)| start..start + piece.len())
        .collect();
    
    let mut chars = value.bytes().enumerate()
        .filter(|(index, _)| ! indexes.iter().any(|range| range.contains(index)))
        .map(|(_, byte)| char::from(byte));
    
    // get first number from left to right
    
    let mut result = chars.find_map(|current| current.to_digit(10)).map(i64::from)?;
    
    while let Some(current) = chars.next() {
        
        if let Some(digit) = current.to_digit(10).map(i64::from) {
            result = result.checked_mul(10)?.checked_add(digit)?;
            continue;
        }
        
        // if next to a digit is a dot and next to the dot is another digit, abort
        if current == '.' && chars.next().filter(char::is_ascii_digit).is_some() {
            return None;
        }
        
        break;
        
    }
    
    Some(result)
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
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(10));
        }
        
        #[test]
        fn unclean() {
            // setup
            
            let value = "[Imaginary] Fictional 86 - 10 [720p]";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(86));
        }
        
        #[test]
        fn zero() {
            // setup
            
            let value = "[Non-existent] Made up - 0 [720p]";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(0));
        }
        
        #[test]
        fn negative() {
            // setup
            
            let value = "[Non-existent] Made up - -17 [720p]";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(17));
        }
        
        #[test]
        fn decimal() {
            // setup
            
            let value = "[Non-existent] Made up - 12.5 [720p]";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn dot() {
            // setup
            
            let value = "[Non-existent] Made up - .24 [720p]";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(24));
        }
        
        #[test]
        fn whitespace() {
            // setup
            
            let value = "[]  - 2 4 []";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(2));
        }
        
        #[test]
        fn big() {
            // setup
            
            let value = "[]  - 42949672 []";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(42949672));
        }
        
        #[test]
        fn overflow() {
            // setup
            
            let value = "[]  - 90223372036854775807 []";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn non_ascii() {
            // setup
            
            let value = "[test] y̆😊aa - 20 [720p]";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, Some(20));
        }
        
        #[test]
        fn empty() {
            // setup
            
            let value = "";
            
            // operation
            
            let output = get(value, &[]);
            
            // control
            
            assert_eq!(output, None);
        }
        
        #[test]
        fn valid_pieces() {
            // setup
            
            let value = "[Imaginary] Fictional 86 - 10 [720p]";
            
            // operation
            
            let output = get(value, &["Imaginary", "Fictional 86", "720p"]);
            
            // control
            
            assert_eq!(output, Some(10));
        }
        
        #[test]
        fn invalid_pieces() {
            // setup
            
            let value = "[Imaginary] Fictional 86 - 10 [720p]";
            
            // operation
            
            let output = get(value, &["Non-existent", "Made up", "74"]);
            
            // control
            
            assert_eq!(output, Some(86));
        }
        
    }
    
}
