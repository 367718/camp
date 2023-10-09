use std::{
    io,
    os::raw::*,
    ptr,
};

use super::{ ffi, Connection };

pub struct Request {
    pub handle: ffi::HINTERNET,
}

impl Request {
    
    // -------------------- constructors --------------------
    
    
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
    
    
    // -------------------- mutators --------------------
    
    
    pub fn send(&mut self) -> io::Result<()> {
        unsafe {
            
            let result = ffi::WinHttpSendRequest(
                self.handle,
                ffi::WINHTTP_NO_ADDITIONAL_HEADERS,
                0,
                ffi::WINHTTP_NO_REQUEST_DATA,
                0,
                0,
                0,
            );
            
            if result == 0 {
                return Err(io::Error::last_os_error());
            }
            
        }
        
        Ok(())
    }
    
    pub fn receive(&mut self) -> io::Result<Vec<u8>> {
        unsafe {
            
            let result = ffi::WinHttpReceiveResponse(
                self.handle,
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(io::Error::last_os_error());
            }
            
        }
        
        let mut response = Vec::new();
        let mut message_size = 0;
        
        loop {
            
            unsafe {
                
                let result = ffi::WinHttpQueryDataAvailable(
                    self.handle,
                    &mut message_size,
                );
                
                if result == 0 {
                    return Err(io::Error::last_os_error());
                }
                
            }
            
            if message_size == 0 {
                break;
            }
            
            let mut current: Vec<u8> = vec![0; message_size as usize];
            let mut bytes = 0;
            
            unsafe {
                
                let result = ffi::WinHttpReadData(
                    self.handle,
                    current.as_mut_ptr().cast::<c_void>(),
                    message_size,
                    &mut bytes,
                );
                
                if result == 0 {
                    return Err(io::Error::last_os_error());
                }
                
            }
            
            response.extend_from_slice(&current[..bytes as usize]);
            
        }
        
        Ok(response)
    }
    
}

impl Drop for Request {
    
    fn drop(&mut self) {
        unsafe {
            
            ffi::WinHttpCloseHandle(self.handle);
            
        }
    }
    
}
