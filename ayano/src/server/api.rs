use std::{
    io::{ self, Error },
    ptr,
};

use crate::sys;

pub struct Api;

impl Api {
    
    pub fn init() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        // perform initialization for applications that use the HTTP server API
        let flags = sys::HTTP_INITIALIZE_SERVER;
        
        unsafe {
            
            let result = sys::HttpInitialize(
                version,
                flags,
                ptr::null_mut(), // reserved
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
        // release all resources used by server applications
        let flags = sys::HTTP_INITIALIZE_SERVER;
        
        unsafe {
            
            sys::HttpTerminate(
                flags,
                ptr::null_mut(), // reserved
            );
            
        }
    }
    
}
