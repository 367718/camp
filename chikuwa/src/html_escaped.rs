use std::io::{ Result, Error, ErrorKind, Read };

pub struct HtmlEscaped<'c> {
    content: &'c [u8],
}

impl<'c> From<&'c [u8]> for HtmlEscaped<'c> {
    
    fn from(content: &'c [u8]) -> Self {
        Self { content }
    }
    
}

impl<'c> Read for HtmlEscaped<'c> {
    
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        
        let Some((byte, rest)) = self.content.split_first() else {
            return Ok(0);
        };
        
        if let Some(escaped) = escape(*byte) {
            
            if buf.len() < escaped.len() {
                return Err(Error::new(ErrorKind::InvalidInput, "Buffer too small"));
            }
            
            buf[..escaped.len()].copy_from_slice(escaped);
            
            self.content = rest;
            
            Ok(escaped.len())
            
        } else {
            
            if buf.is_empty() {
                return Err(Error::new(ErrorKind::InvalidInput, "Buffer too small"));
            }
            
            buf[0] = *byte;
            
            self.content = rest;
            
            Ok(1)
            
        }
        
    }
    
}

fn escape(byte: u8) -> Option<&'static [u8]> {
    // https://www.w3.org/International/questions/qa-escapes#use
    match byte {
        b'&' => Some(b"&amp;"),
        b'<' => Some(b"&lt;"),
        b'>' => Some(b"&gt;"),
        b'"' => Some(b"&quot;"),
        b'\'' => Some(b"&apos;"),
        _ => None,
    }
}
