use std::io::{ self, Error, ErrorKind };

pub fn win_string(content: &str) -> io::Result<Vec<u16>> {
    if content.contains('\0') {
        return Err(Error::new(ErrorKind::InvalidInput, "Input contains an interior null byte"));
    }
    
    // a UTF-8 string byte length is guaranteed to be greater than or equal to its UTF-16 code-unit length
    // an additional slot is reserved for the null terminator
    let mut result = Vec::with_capacity(content.len() + 1);
    
    result.extend(content.encode_utf16());
    result.push(0);
    
    Ok(result)
}
