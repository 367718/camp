use std::{
    io::{ self, Error },
    ptr,
};

use super::{ ffi, Handle, Connection };

pub struct Request {
    pub handle: Handle,
}

impl Request {
    
    pub fn new(connection: &Connection, path: &str, secure: bool) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let flags = if secure {
            ffi::WINHTTP_FLAG_SECURE
        } else {
            0
        };
        
        let handle = unsafe {
            
            let result = ffi::WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(),
                chikuwa::WinString::from(path).as_ptr(),
                ptr::null(),
                ffi::WINHTTP_NO_REFERER,
                ffi::WINHTTP_DEFAULT_ACCEPT_TYPES,
                flags,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            Handle::new(result)
            
        };
        
        // -------------------- send --------------------
        
        unsafe {
            
            let result = ffi::WinHttpSendRequest(
                handle.as_raw(),
                ffi::WINHTTP_NO_ADDITIONAL_HEADERS,
                0,
                ptr::null_mut(),
                0,
                0,
                0,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(Self {
            handle,
        })
    }
    
}
