// https://www.w3.org/International/questions/qa-escapes#use

use std::io::{ self, Write };

pub fn escape_html(content: &[u8], writer: &mut impl Write) -> io::Result<()> {
    let mut previous_position = 0;
    
    for (current_position, &byte) in content.iter().enumerate() {
        if matches!(byte, b'&' | b'<' | b'>' | b'"' | b'\'') {
            
            if previous_position < current_position {
                writer.write_all(&content[previous_position..current_position])?;
            }
            
            let replacement: &[u8] = match byte {
                b'&' => b"&amp;",
                b'<' => b"&lt;",
                b'>' => b"&gt;",
                b'"' => b"&quot;",
                b'\'' => b"&apos;",
                _ => unreachable!(),
            };
            
            writer.write_all(replacement)?;
            
            previous_position = current_position + 1;
            
        }
    }
    
    if previous_position < content.len() {
        writer.write_all(&content[previous_position..])?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn empty() {
        // setup
        
        let content = "";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, content.as_bytes());
    }
    
    #[test]
    fn no_replacement() {
        // setup
        
        let content = "-(d))m*Qz¡dEpM¡Ez-M2";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, content.as_bytes());
    }
    
    #[test]
    fn single_first() {
        // setup
        
        let content = "&yL6LN4IW1RqEHNe0";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, b"&amp;yL6LN4IW1RqEHNe0");
    }
    
    #[test]
    fn single_last() {
        // setup
        
        let content = "yL6LN4IW1RqEHNe0>";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, b"yL6LN4IW1RqEHNe0&gt;");
    }
    
    #[test]
    fn multiple() {
        // setup
        
        let content = "yL6LN4IW&1RqEH<N<e>0";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, b"yL6LN4IW&amp;1RqEH&lt;N&lt;e&gt;0");
    }
    
    #[test]
    fn full() {
        // setup
        
        let content = "<&<>";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, b"&lt;&amp;&lt;&gt;");
    }
    
    #[test]
    fn emoji() {
        // setup
        
        let content = "<&<🔌>";
        let mut writer = Vec::new();
        
        // operation
        
        let output = escape_html(content.as_bytes(), &mut writer);
        
        // control
        
        assert!(output.is_ok());
        assert_eq!(writer, "&lt;&amp;&lt;🔌&gt;".as_bytes());
    }
    
}
