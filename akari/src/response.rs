use std::io::{ self, Read };

use super::{ ffi, Handle };

pub struct Response {
    handle: Handle,
}

impl Response {
    
    pub fn new(handle: Handle) -> io::Result<Self> {
        ffi::receive_response(&handle)?;
        
        Ok(Self {
            handle,
        })
    }
    
    pub fn content_length(&self) -> io::Result<usize> {
        ffi::query_headers(&self.handle)
    }
    
}

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        ffi::read_data(&self.handle, buf)
    }
    
}
