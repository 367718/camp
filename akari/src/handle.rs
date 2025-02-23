use std::os::{
    raw::*,
    windows::io::RawHandle,
};

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    fn WinHttpCloseHandle(
        h_internet: RawHandle,
    ) -> c_int; // BOOL
    
}

pub struct HttpHandle {
    inner: RawHandle,
}

impl HttpHandle {
    
    pub fn new(raw: RawHandle) -> Self {
        Self {
            inner: raw,
        }
    }
    
    pub fn as_raw(&self) -> RawHandle {
        self.inner
    }
    
}

impl Drop for HttpHandle {
    
    fn drop(&mut self) {
        unsafe {
            
            WinHttpCloseHandle(
                self.inner,
            );
            
        }
    }
    
}
