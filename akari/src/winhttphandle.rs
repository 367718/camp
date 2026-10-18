use std::os::raw::*;

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    fn WinHttpCloseHandle(
        h_internet: *mut c_void, // HINTERNET
    ) -> c_int; // BOOL
    
}

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
            
            WinHttpCloseHandle(
                self.0,
            );
            
        }
    }
    
}
