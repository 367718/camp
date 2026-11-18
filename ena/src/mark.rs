use std::{
    ffi::{ OsStr, OsString },
    fs,
    io::{ self, Error, ErrorKind },
    path::Path,
};

use crate::sys;

pub fn is_marked(root: &Path, relative: &Path, flag: impl AsRef<OsStr>) -> io::Result<bool> {
    let stream = build_stream(root.as_os_str(), relative.as_os_str(), flag.as_ref())?;
    let stream_wide = chikuwa::win_os_string(&stream)?;
    
    // SAFETY: `stream_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
    let result = unsafe {
        
        sys::GetFileAttributesW(
            stream_wide.as_ptr(),
        )
        
    };
    
    // `GetFileAttributesW` succeeded: file is marked
    if result != sys::INVALID_FILE_ATTRIBUTES {
        return Ok(true);
    }
    
    let error = Error::last_os_error();
    
    // `GetFileAttributesW` failed with "not found": assume file is unmarked
    if error.kind() == ErrorKind::NotFound {
        return Ok(false);
    }
    
    Err(error)
}

pub fn toggle(root: &Path, relative: &Path, flag: impl AsRef<OsStr>) -> io::Result<()> {
    let stream = build_stream(root.as_os_str(), relative.as_os_str(), flag.as_ref())?;
    
    // if `flag` existed, the job is already done
    // if `flag` didn't exist, create it
    match fs::remove_file(&stream) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => fs::write(&stream, ""),
        Err(error) => Err(error),
    }
}

fn build_stream(root: &OsStr, relative: &OsStr, flag: &OsStr) -> io::Result<OsString> {
    let bytes = flag.as_encoded_bytes();
    
    if bytes.is_empty() || bytes.eq_ignore_ascii_case(b"$DATA") || bytes.iter().any(|&byte| matches!(byte, b':' | b'\\' | b'/' | 0)) {
        return Err(Error::new(ErrorKind::InvalidInput, "Invalid file flag"));
    }
    
    // avoid `\\` in verbatim `\\?\` paths when `root` is a drive root like `\\?\C:\`
    let needs_separator = ! root
        .as_encoded_bytes()
        .ends_with(br"\");
    
    let mut stream = OsString::with_capacity(root.len() + usize::from(needs_separator) + relative.len() + 1 + flag.len());
    
    stream.push(root);
    
    if needs_separator {
        stream.push(r"\");
    }
    
    stream.push(relative);
    stream.push(":");
    stream.push(flag);
    
    Ok(stream)
}
