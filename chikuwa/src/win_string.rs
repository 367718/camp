use std::{
    io::{ self, Error, ErrorKind },
    iter,
};

pub fn win_string(content: &str) -> io::Result<Vec<u16>> {
    if content.contains('\0') {
        return Err(Error::new(ErrorKind::InvalidInput, "Input contains an interior null byte"));
    }
    
    let result = content.encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    
    Ok(result)
}
