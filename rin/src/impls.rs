use std::io::{ self, Error, ErrorKind };

pub trait ParamFromValue<'c>: Sized {
    
    fn param_from_value(bytes: &'c [u8]) -> io::Result<Self>;
    
}

// -------------------- &str --------------------

impl<'c> ParamFromValue<'c> for &'c str {
    
    fn param_from_value(value: &'c [u8]) -> io::Result<Self> {
        str::from_utf8(value)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))
    }
    
}

// -------------------- u64 --------------------

impl<'c> ParamFromValue<'c> for u64 {
    
    fn param_from_value(value: &'c [u8]) -> io::Result<Self> {
        // TODO: replace with u64::from_ascii in the future (https://github.com/rust-lang/rust/issues/134821)
        let interim_str = <&str>::param_from_value(value)?;
        interim_str.parse()
            .map_err(|error| Error::new(ErrorKind::InvalidData, error))
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod to_str {
        
        // letters
        // numbers
        // emoji
        // empty
        // encoding_error
        
        use super::*;
        
        #[test]
        fn letters() {
            // setup
            
            let value = b"placeholder";
            
            // operation
            
            let output = <&str>::param_from_value(value);
            
            // control
            
            let output = output.unwrap();
            
            assert_eq!(output, "placeholder");
        }
        
        #[test]
        fn numbers() {
            // setup
            
            let value = b"123";
            
            // operation
            
            let output = <&str>::param_from_value(value);
            
            // control
            
            let output = output.unwrap();
            
            assert_eq!(output, "123");
        }
        
        #[test]
        fn emoji() {
            // setup
            
            let value = "te🔌st";
            
            // operation
            
            let output = <&str>::param_from_value(value.as_bytes());
            
            // control
            
            let output = output.unwrap();
            
            assert_eq!(output, "te🔌st");
        }
        
        #[test]
        fn empty() {
            // setup
            
            let value = b"";
            
            // operation
            
            let output = <&str>::param_from_value(value);
            
            // control
            
            let output = output.unwrap();
            
            assert_eq!(output, "");
        }
        
        #[test]
        fn encoding_error() {
            // setup
            
            // 0xC3 => start of a two-byte sequence
            // 0x28 => control character, outside required range for a continuation byte (0x80-0xBF)
            let value = &[0xC3, 0x28];
            
            // operation
            
            let output = <&str>::param_from_value(value);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    #[cfg(test)]
    mod to_u64 {
        
        // letters
        // numbers
        // emoji
        // empty
        // encoding_error
        
        use super::*;
        
        #[test]
        fn letters() {
            // setup
            
            let value = b"placeholder";
            
            // operation
            
            let output = u64::param_from_value(value);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn numbers() {
            // setup
            
            let value = b"123";
            
            // operation
            
            let output = u64::param_from_value(value);
            
            // control
            
            let output = output.unwrap();
            
            assert_eq!(output, 123);
        }
        
        #[test]
        fn emoji() {
            // setup
            
            let value = "te🔌st";
            
            // operation
            
            let output = u64::param_from_value(value.as_bytes());
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn empty() {
            // setup
            
            let value = b"";
            
            // operation
            
            let output = u64::param_from_value(value);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn encoding_error() {
            // setup
            
            // 0xC3 => start of a two-byte sequence
            // 0x28 => control character, outside required range for a continuation byte (0x80-0xBF)
            let value = &[0xC3, 0x28];
            
            // operation
            
            let output = u64::param_from_value(value);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
}
