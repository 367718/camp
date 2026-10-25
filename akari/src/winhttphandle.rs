use std::os::raw::*;

use crate::sys;

pub struct WinHttpHandle(*mut c_void);

impl WinHttpHandle {
    
    pub fn new(inner: *mut c_void) -> Self {
        Self(inner)
    }
    
    pub fn as_raw(&self) -> *mut c_void {
        self.0
    }
    
}

impl Drop for WinHttpHandle {
    
    fn drop(&mut self) {
        unsafe {
            
            sys::WinHttpCloseHandle(
                self.0,
            );
            
        }
    }
    
}
