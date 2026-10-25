use std::{
    io::{ self, Error },
    os::raw::*,
    ptr,
};

use crate::sys;

pub struct Queue { 
    pub(crate) handle: *mut c_void, // HANDLE
}

impl Queue {
    
    pub fn new() -> io::Result<Self> {
        let mut handle = ptr::null_mut();
        
        unsafe {
            
            let result = sys::HttpCreateRequestQueue(
                sys::HTTPAPI_VERSION_2,
                ptr::null(),
                ptr::null_mut(),
                0,
                &raw mut handle,
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        }
        
        Ok(Self { handle })
    }
    
}

impl Drop for Queue {
    
    fn drop(&mut self) {
        unsafe {
            
            sys::HttpCloseRequestQueue(
                self.handle,
            );
            
        }
    }
    
}
