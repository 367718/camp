use std::{
    io,
    mem,
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
        
        unsafe {
            
            let mut content_length: c_ulong = 0;
            
            #[allow(clippy::cast_possible_truncation)]
            let mut bytes = mem::size_of::<c_ulong>() as c_ulong;
            
            ffi::WinHttpQueryHeaders(
                self.request.handle,
                ffi::WINHTTP_QUERY_CONTENT_LENGTH | ffi::WINHTTP_QUERY_FLAG_NUMBER,
                ffi::WINHTTP_HEADER_NAME_BY_INDEX,
                ptr::addr_of_mut!(content_length).cast::<c_void>(),
                ptr::addr_of_mut!(bytes),
                ffi::WINHTTP_NO_HEADER_INDEX,
            );
            
            payload.reserve_exact(content_length as usize);
            
        }
        
        let mut buffer = [0; CONNECTION_BUFFER_SIZE as usize];
        let mut amount_read: c_ulong = 0;
        
        loop {
            
            unsafe {
                
                let result = ffi::WinHttpReadData(
                    self.request.handle,
                    buffer.as_mut_ptr().cast::<c_void>(),
                    CONNECTION_BUFFER_SIZE,
                    &mut amount_read,
                );
                
                if result == 0 {
                    return Err(io::Error::last_os_error());
                }
                
                if amount_read == 0 {
                    break;
                }
                
            }
            
            payload.extend_from_slice(&buffer[..amount_read as usize]);
            
        }
        
        Ok(payload)
    }
    
}
