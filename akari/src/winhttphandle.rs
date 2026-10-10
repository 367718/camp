use std::os::{
    raw::*,
    windows::io::{ RawHandle, AsRawHandle },
};

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    fn WinHttpCloseHandle(
        h_internet: RawHandle,
    ) -> c_int; // BOOL
    
}

pub struct WinHttpHandle(RawHandle);

impl From<RawHandle> for WinHttpHandle {
    
    fn from(inner: RawHandle) -> Self {
        Self(inner)
    }
    
}

impl AsRawHandle for WinHttpHandle {
    
    fn as_raw_handle(&self) -> RawHandle {
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
