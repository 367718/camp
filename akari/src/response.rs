use std::{
    io,
    os::raw::*,
    ptr,
};

use super::{ ffi, Request };

const CONNECTION_BUFFER_SIZE: c_ulong = 8 * 1024;

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
        let mut payload = Vec::new();
        let mut buffer = [0; CONNECTION_BUFFER_SIZE as usize];
        
        loop {
            
            let mut amount_read = 0;
            
            unsafe {
                
                let result = ffi::WinHttpReadData(
                    self.request.handle,
                    buffer.as_mut_ptr().cast::<c_void>(),
                    CONNECTION_BUFFER_SIZE,
                    &mut amount_read,
                );
                
                if amount_read == 0 {
                    break;
                }
                
                if result == 0 {
                    return Err(io::Error::last_os_error());
                }
                
            }
            
            payload.extend_from_slice(&buffer[..amount_read as usize]);
            
        }
        
        Ok(payload)
    }
    
}
