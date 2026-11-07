use std::{
    io::{ self, Error },
    ptr,
};

use crate::sys;

pub(super) struct Api;

impl Api {
    
    pub(super) fn init() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        // perform initialization for applications that use the HTTP server API
        let flags = sys::HTTP_INITIALIZE_SERVER;
        
        // SAFETY:
        // 1. `version` is a valid `HTTPAPI_VERSION` struct
        // 2. `flags` is a valid option defined by the http.sys API
        // 3. `p_reserved` as `null` is required by the http.sys API
        let result = unsafe {
            
            sys::HttpInitialize(
                version,
                flags,
                ptr::null_mut(), // reserved
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(Self {})
    }
    
}

impl Drop for Api {
    
    fn drop(&mut self) {
        // release all resources used by server applications
        let flags = sys::HTTP_INITIALIZE_SERVER;
        
        // SAFETY:
        // 1. `flags` matches the flags previously specified in `HttpInitialize`
        // 2. `p_reserved` as `null` is required by the http.sys API
        // 3. `drop` runs only once, when the struct goes out of scope, preventing double-terminate bugs
        unsafe {
            
            sys::HttpTerminate(
                flags,
                ptr::null_mut(), // reserved
            )
            
        };
    }
    
}
