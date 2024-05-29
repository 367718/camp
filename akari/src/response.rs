use std::{
    io::{ self, Read, Error },
    mem,
    os::raw::*,
    ptr,
};

use super::{ ffi, Handle, Request };

pub struct Response {
    handle: Handle,
}

impl Response {
    
    pub fn new(request: Request) -> io::Result<Self> {
        let handle = request.handle;
        
        unsafe {
            
            let result = ffi::WinHttpReceiveResponse(
                handle.as_raw(),
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(Self {
            handle,
        })
    }
    
    pub fn content_length(&self) -> usize {
        let mut content_length: c_ulong = 0;
        
        #[allow(clippy::cast_possible_truncation)]
        let mut bytes = mem::size_of::<c_ulong>() as c_ulong;
        
        unsafe {
            
            ffi::WinHttpQueryHeaders(
                self.handle.as_raw(),
                ffi::WINHTTP_QUERY_CONTENT_LENGTH | ffi::WINHTTP_QUERY_FLAG_NUMBER,
                ffi::WINHTTP_HEADER_NAME_BY_INDEX,
                ptr::from_mut(&mut content_length).cast::<c_void>(),
                &mut bytes,
                ffi::WINHTTP_NO_HEADER_INDEX,
            );
            
        }
        
        content_length as usize
    }
    
}

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: c_ulong = 0;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = buf.len() as c_ulong;
        
        unsafe {
            
            let result = ffi::WinHttpReadData(
                self.handle.as_raw(),
                ptr::from_mut(buf).cast::<c_void>(),
                bytes,
                &mut amount_read,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(amount_read as usize)
    }
    
}
