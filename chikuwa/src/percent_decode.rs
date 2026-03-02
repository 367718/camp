// https://developer.mozilla.org/en-US/docs/Glossary/Percent-encoding

use std::io::{ self, Error, ErrorKind, Write };

pub fn percent_decode(content: &[u8], mut writer: impl Write) -> io::Result<()> {
    let mut previous_position = 0;
    
    let mut bytes = content.iter()
        .copied()
        .enumerate();
    
    while let Some((current_position, byte)) = bytes.next() {
        
        if ! matches!(byte, b'%' | b'+') {
            continue;
        }
        
        // skipped chunk of unencoded characters
        if previous_position < current_position {
            writer.write_all(&content[previous_position..current_position])?;
        }
        
        if byte == b'%' {
            
            let high = bytes.next()
                .and_then(|(_, byte)| decode_hex(byte))
                .ok_or(Error::from(ErrorKind::InvalidInput))?;
            
            let low = bytes.next()
                .and_then(|(_, byte)| decode_hex(byte))
                .ok_or(Error::from(ErrorKind::InvalidInput))?;
            
            // decoded character
            writer.write_all(&[high << 4 | low])?;
            
            previous_position = current_position + 3;
            
        } else {
            
            // decoded '+' character
            writer.write_all(b" ")?;
            
            previous_position = current_position + 1;
            
        }
        
    }
    
    // remaining unencoded characters
    if previous_position < content.len() {
        writer.write_all(&content[previous_position..])?;
    }
    
    Ok(())
}

fn decode_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0' ..= b'9' => Some(byte - b'0'),
        b'a' ..= b'f' => Some(byte - b'a' + 10),
        b'A' ..= b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn single() {
        // setup
        
        let content = "place%21holder";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "place!holder".as_bytes());
    }
    
    #[test]
    fn single_first() {
        // setup
        
        let content = "%21placeholder";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "!placeholder".as_bytes());
    }
    
    #[test]
    fn single_last() {
        // setup
        
        let content = "placeholder%21";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "placeholder!".as_bytes());
    }
    
    #[test]
    fn multiple() {
        // setup
        
        let content = "place%5Bholder%5D";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(String::from_utf8_lossy(&writer), "place[holder]");
        assert_eq!(writer, "place[holder]".as_bytes());
    }
    
    #[test]
    fn full() {
        // setup
        
        let content = "%5B%21%5D";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "[!]".as_bytes());
    }
    
    #[test]
    fn white_space() {
        // setup
        
        let content = "placeholder+test";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "placeholder test".as_bytes());
    }
    
    #[test]
    fn white_space_plus_symbols() {
        // setup
        
        let content = "placeholder+%5B%21%5D+test";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "placeholder [!] test".as_bytes());
    }
    
    #[test]
    fn emoji() {
        // setup
        
        let content = "place%21🔌holder";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "place!🔌holder".as_bytes());
    }
    
    #[test]
    fn no_replacement() {
        // setup
        
        let content = "placeholder";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, content.as_bytes());
    }
    
    #[test]
    fn empty() {
        // setup
        
        let content = "";
        let mut writer = Vec::new();
        
        // operation
        
        let output = percent_decode(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, content.as_bytes());
    }
    
}
