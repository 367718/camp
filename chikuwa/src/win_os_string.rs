use std::{
    ffi::OsStr,
    io::{ self, Error, ErrorKind },
    iter,
    os::windows::ffi::OsStrExt,
};

pub fn win_os_string(content: &OsStr) -> io::Result<Vec<u16>> {
    if content.as_encoded_bytes().contains(&b'\0') {
        return Err(Error::new(ErrorKind::InvalidInput, "Input contains an interior null byte"));
    }
    
    let result = content
        .encode_wide()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    
    Ok(result)
}
