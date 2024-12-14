pub fn escape_html(content: &[u8]) -> impl Iterator<Item = &[u8]> {
    content.windows(1).map(|current| match current {
        
        // https://www.w3.org/International/questions/qa-escapes#use
        
        b"&" => b"&amp;".as_slice(),
        b"<" => b"&lt;".as_slice(),
        b">" => b"&gt;".as_slice(),
        
        b"\"" => b"&quot;".as_slice(),
        b"'" => b"&apos;".as_slice(),
        
        _ => current,
        
    })
}
