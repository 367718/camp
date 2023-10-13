use std::{
    io::{ self, Read },
    mem,
    os::raw::*,
    ptr,
};

use super::{ ffi, Connection };

pub struct Payload {
    handle: ffi::HINTERNET,
}

impl Payload {
    
    // -------------------- constructors --------------------
    
    
    pub fn new(connection: &Connection, path: &str, secure: bool) -> io::Result<Self> {
        // ---------- handle ----------
        
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
        
        // ---------- send ----------
        
        unsafe {
            
            let result = ffi::WinHttpSendRequest(
                handle,
                ffi::WINHTTP_NO_ADDITIONAL_HEADERS,
                0,
                ffi::WINHTTP_NO_REQUEST_DATA,
                0,
                0,
                0,
            );
            
            if result == 0 {
                ffi::WinHttpCloseHandle(handle);
                return Err(io::Error::last_os_error());
            }
            
        }
        
        // ---------- receive ----------
        
        unsafe {
            
            let result = ffi::WinHttpReceiveResponse(
                handle,
                ptr::null_mut(),
            );
            
            if result == 0 {
                ffi::WinHttpCloseHandle(handle);
                return Err(io::Error::last_os_error());
            }
            
        }
        
        Ok(Self { handle })
    }
    
    
    // -------------------- constructors --------------------
    
    
    pub fn content_length(&self) -> usize {
        let mut result: c_ulong = 0;
        
        unsafe {
            
            #[allow(clippy::cast_possible_truncation)]
            let mut bytes = mem::size_of::<c_ulong>() as c_ulong;
            
            ffi::WinHttpQueryHeaders(
                self.handle,
                ffi::WINHTTP_QUERY_CONTENT_LENGTH | ffi::WINHTTP_QUERY_FLAG_NUMBER,
                ffi::WINHTTP_HEADER_NAME_BY_INDEX,
                ptr::addr_of_mut!(result).cast::<c_void>(),
                ptr::addr_of_mut!(bytes),
                ffi::WINHTTP_NO_HEADER_INDEX,
            );
            
        }
        
        result as usize
    }
    
}

impl Read for Payload {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: c_ulong = 0;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = buf.len() as c_ulong;
        
        unsafe {
            
            let result = ffi::WinHttpReadData(
                self.handle,
                buf.as_mut_ptr().cast::<c_void>(),
                bytes,
                &mut amount_read,
            );
            
            if result == 0 {
                return Err(io::Error::last_os_error());
            }
            
        }
        
        Ok(amount_read as usize)
    }
    
}

impl Drop for Payload {
    
    fn drop(&mut self) {
        unsafe {
            
            ffi::WinHttpCloseHandle(self.handle);
            
        }
    }
    
}
