use std::{
    io,
    os::raw::*,
};

use super::ffi;

const DNS_RESOLUTION_TIMEOUT: c_int = 15_000;
const CONNECTION_TIMEOUT: c_int = 15_000;
const SEND_TIMEOUT: c_int = 15_000;
const RECEIVE_TIMEOUT: c_int = 15_000;

pub struct Session {
    pub handle: ffi::HINTERNET,
}

impl Session {
    
    pub fn new() -> io::Result<Self> {
        let handle = unsafe {
            
            let result = ffi::WinHttpOpen(
                chikuwa::WinString::from(env!("CARGO_PKG_NAME")).as_ptr(),
                ffi::WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                ffi::WINHTTP_NO_PROXY_NAME,
                ffi::WINHTTP_NO_PROXY_BYPASS,
                0,
            );
            
            if result.is_null() {
                return Err(io::Error::last_os_error());
            }
            
            result
            
        };
        
        unsafe {
            
            let result = ffi::WinHttpSetTimeouts(
                handle,
                DNS_RESOLUTION_TIMEOUT,
                CONNECTION_TIMEOUT,
                SEND_TIMEOUT,
                RECEIVE_TIMEOUT,
            );
            
            if result == 0 {
                let error = Err(io::Error::last_os_error());
                ffi::WinHttpCloseHandle(handle);
                return error;
            }
            
        }
        
        Ok(Self { handle })
    }
    
}

impl Drop for Session {
    
    fn drop(&mut self) {
        unsafe {
            
            ffi::WinHttpCloseHandle(self.handle);
            
        }
    }
    
}
