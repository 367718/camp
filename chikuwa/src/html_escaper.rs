pub struct HtmlEscaper<'e> {
    content: &'e [u8],
}

impl<'e> From<&'e [u8]> for HtmlEscaper<'e> {
    
    fn from(content: &'e [u8]) -> Self {
        Self { content }
    }
    
}

impl<'e> Iterator for HtmlEscaper<'e> {
    
    type Item = &'e [u8];
    
    fn next(&mut self) -> Option<Self::Item> {
        
        let (current, rest) = self.content.split_at_checked(1)?;
        self.content = rest;
        
        // https://www.w3.org/International/questions/qa-escapes#use
        Some(match current {
            
            b"&" => b"&amp;",
            b"<" => b"&lt;",
            b">" => b"&gt;",
            
            b"\"" => b"&quot;",
            b"'" => b"&apos;",
            
            _ => current,
            
        })
        
    }
    
}
