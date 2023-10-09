use std::{
    io,
    os::raw::*,
    ptr,
};

use super::{ ffi, Request };

pub struct Response {
    request: Request,
}

impl Response {
    
    // -------------------- constructors --------------------
    
    
    pub fn new(request: Request) -> io::Result<Self> {
        unsafe {
            
            let result = ffi::WinHttpSendRequest(
                request.handle,
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
        
        unsafe {
            
            let result = ffi::WinHttpReceiveResponse(
                request.handle,
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(io::Error::last_os_error());
            }
            
        }
        
        Ok(Self { request })
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn payload(self) -> io::Result<Vec<u8>> {
        let mut response = Vec::new();
        
        loop {
            
            let mut message_size = 0;
            
            unsafe {
                
                let result = ffi::WinHttpQueryDataAvailable(
                    self.request.handle,
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
            let mut amount_read = 0;
            
            unsafe {
                
                let result = ffi::WinHttpReadData(
                    self.request.handle,
                    current.as_mut_ptr().cast::<c_void>(),
                    message_size,
                    &mut amount_read,
                );
                
                if result == 0 {
                    return Err(io::Error::last_os_error());
                }
                
            }
            
            response.extend_from_slice(&current[..amount_read as usize]);
            
        }
        
        Ok(response)
    }
    
}
