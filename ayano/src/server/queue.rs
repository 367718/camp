use std::{
    io::{ self, Error },
    os::raw::c_void,
    ptr,
};

use crate::sys;

pub(super) struct Queue { 
    pub(super) handle: *mut c_void, // HANDLE
}

impl Queue {
    
    pub(super) fn new() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        let mut handle = ptr::null_mut();
        
        // SAFETY:
        // 1. `version` is a valid `HttpApiVersion` struct
        // 2. `p_name` as `null` is valid and selects standard http.sys defaults
        // 3. `p_security_attributes` as `null` is valid and selects standard http.sys defaults
        // 4. `flags` as `0` is valid and selects standard http.sys defaults
        // 5. `&raw mut handle` is a live, properly aligned pointer, valid for writing a `HANDLE`
        let result = unsafe {
            
            sys::HttpCreateRequestQueue(
                version,
                ptr::null(), // queue cannot be accessed by name
                ptr::null(), // disable child process handle inheritance
                0, // create a new, standard worker queue
                &raw mut handle,
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(Self { handle })
    }
    
}

impl Drop for Queue {
    
    fn drop(&mut self) {
        // SAFETY:
        // 1. `self.handle` is guaranteed to be a valid, non-null queue handle initialized by the `HttpCreateRequestQueue` function
        // 2. ownership of the raw handle belongs exclusively to this `Queue` instance
        // 3. `drop` runs only once, when the struct goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::HttpCloseRequestQueue(
                self.handle,
            )
            
        };
    }
    
}
