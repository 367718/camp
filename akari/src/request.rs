use std::{
    io,
    ptr,
};

use super::{ ffi, Connection };

pub struct Request {
    pub handle: ffi::HINTERNET,
}

impl Request {
    
    pub fn new(connection: &Connection, path: &str, secure: bool) -> io::Result<Self> {
        let handle = unsafe {
            
            let flags = if secure {
                ffi::WINHTTP_FLAG_SECURE
            } else {
                0
            };
            
            let result = ffi::WinHttpOpenRequest(
                connection.handle,
                chikuwa::WinString::from("GET").as_ptr(),
                chikuwa::WinString::from(path).as_ptr(),
                ptr::null(),
                ffi::WINHTTP_NO_REFERER,
                ffi::WINHTTP_DEFAULT_ACCEPT_TYPES,
                flags,
            );
            
            if result.is_null() {
                return Err(io::Error::last_os_error());
            }
            
            result
            
        };
        
        Ok(Self { handle })
    }
    
}

impl Drop for Request {
    
    fn drop(&mut self) {
        unsafe {
            
            ffi::WinHttpCloseHandle(self.handle);
            
        }
    }
    
}
