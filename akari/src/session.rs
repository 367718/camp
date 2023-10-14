use std::{
    io,
    mem,
    os::raw::*,
    ptr,
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
        // ---------- handle ----------
        
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
        
        // ---------- timeouts ----------
        
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
        
        // ---------- http version ----------
        
        unsafe {
            
            let mut version = ffi::WINHTTP_PROTOCOL_FLAG_HTTP2;
            
            #[allow(clippy::cast_possible_truncation)]
            let bytes = mem::size_of::<c_ulong>() as c_ulong;
            
            let result = ffi::WinHttpSetOption(
                handle,
                ffi::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                ptr::addr_of_mut!(version).cast::<c_void>(),
                bytes,
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
