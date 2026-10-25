use std::{
    io::{ self, Error },
    ptr,
};

use crate::sys;

pub struct Api;

impl Api {
    
    pub fn init() -> io::Result<Self> {
        unsafe {
            
            let result = sys::HttpInitialize(
                sys::HTTPAPI_VERSION_2,
                sys::HTTP_INITIALIZE_SERVER,
                ptr::null_mut(),
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        };
        
        Ok(Self {})
    }
    
}

impl Drop for Api {
    
    fn drop(&mut self) {
        unsafe {
            
            sys::HttpTerminate(
                sys::HTTP_INITIALIZE_SERVER,
                ptr::null_mut(),
            );
            
        }
    }
    
}
