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
        
        let current = &[*byte];
        
        // https://www.w3.org/International/questions/qa-escapes#use
        let escaped: &[u8] = match current {
            b"&" => b"&amp;",
            b"<" => b"&lt;",
            b">" => b"&gt;",
            b"\"" => b"&quot;",
            b"'" => b"&apos;",
            _ => current,
        };
        
        if buf.len() < escaped.len() {
            return Err(Error::new(ErrorKind::InvalidInput, "Buffer too small"));
        }
        
        buf[..escaped.len()].copy_from_slice(escaped);
        
        self.content = rest;
        
        Ok(escaped.len())
        
    }
    
}
