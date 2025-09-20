// https://developer.mozilla.org/en-US/docs/Glossary/Percent-encoding

use std::io::{ self, Write };

pub fn percent_decode(content: &[u8], writer: &mut impl Write) -> io::Result<()> {
    if ! content.iter().any(|byte| matches!(byte, b'%' | b'+')) {
        return writer.write_all(content);
    }
    
    let mut iter = content.iter();
    
    while let Some(byte) = iter.next() {
        match *byte {
            
            b'%' => {
                
                let Some(left) = iter.next().map(|left| char::from(*left)) else {
                    break;
                };
                
                let Some(right) = iter.next().map(|right| char::from(*right)) else {
                    break;
                };
                
                if let Ok(decoded) = u8::from_str_radix(&format!("{}{}", left, right), 16) {
                    writer.write_all(&[decoded])?;
                }
                
            },
            
            b'+' => writer.write_all(b" ")?,
            
            _ => writer.write_all(&[*byte])?,
            
        }
    }
    
    Ok(())
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
