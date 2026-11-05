use std::{
    io,
    os::raw::*,
    ptr::NonNull,
};

use crate::{ sys, last_winhttp_error };

pub(crate) struct WinHttpHandle(NonNull<c_void>); // HINTERNET

impl WinHttpHandle {
    
    /// # Safety
    ///
    /// if `raw` is non-null, it must be a valid, open `HINTERNET` handle whose exclusive ownership is transferred
    pub(crate) unsafe fn from_raw(raw: *mut c_void) -> io::Result<Self> {
        let inner = NonNull::new(raw)
            .ok_or_else(last_winhttp_error)?;
        
        Ok(Self(inner))
    }
    
    pub(crate) fn as_raw(&self) -> *mut c_void {
        self.0.as_ptr()
    }
    
}


impl Drop for WinHttpHandle {
    
    fn drop(&mut self) {
        
        // SAFETY:
        // 1. `self.0` is guaranteed to be a valid, non-null `HINTERNET` handle initialized by a WinHTTP creation function
        // 2. ownership of the raw handle belongs exclusively to this `WinHttpHandle` instance
        // 3. `drop` runs only once, when the handle goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::WinHttpCloseHandle(
                self.0.as_ptr(),
            );
            
        }
        
    }
    
}
