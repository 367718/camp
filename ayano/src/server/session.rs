use std::io::{ self, Error };

use crate::sys;

pub(super) struct Session {
    pub(super) id: u64, // HTTP_OPAQUE_ID
}

impl Session {
    
    pub(super) fn new() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        let mut id: u64 = 0;
        
        // SAFETY:
        // 1. `version` is a valid `HttpApiVersion` struct
        // 2. `&raw mut id` points to a live, properly aligned `u64` valid for writes
        // 3. `reserved` as `0` is required by the http.sys API
        let result = unsafe {
            
            sys::HttpCreateServerSession(
                version,
                &raw mut id,
                0, // reserved
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(Self { id })
    }
    
}

impl Drop for Session {
    
    fn drop(&mut self) {
        // SAFETY:
        // 1. `self.id` is a valid, active `HTTP_OPAQUE_ID` created by `HttpCreateServerSession` that has not been closed already
        // 2. `drop` runs only once, when the struct goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::HttpCloseServerSession(
                self.id,
            )
            
        };
    }
    
}
