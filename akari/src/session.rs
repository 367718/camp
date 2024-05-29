use std::{
    io::{ self, Error },
    mem,
    os::raw::*,
    ptr,
};

use super::{
    ffi, Handle,
    DNS_RESOLUTION_TIMEOUT_AS_MILLIS, CONNECTION_TIMEOUT_AS_MILLIS,
    SEND_TIMEOUT_AS_MILLIS, RECEIVE_TIMEOUT_AS_MILLIS,
};

pub struct Session {
    pub handle: Handle,
}

impl Session {
    
    pub fn new() -> io::Result<Self> {
        // -------------------- open --------------------
        
        let handle = unsafe {
            
            let result = ffi::WinHttpOpen(
                chikuwa::WinString::from(env!("CARGO_PKG_NAME")).as_ptr(),
                ffi::WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                ffi::WINHTTP_NO_PROXY_NAME,
                ffi::WINHTTP_NO_PROXY_BYPASS,
                0,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            Handle::new(result)
            
        };
        
        // -------------------- timeouts --------------------
        
        unsafe {
            
            let result = ffi::WinHttpSetTimeouts(
                handle.as_raw(),
                DNS_RESOLUTION_TIMEOUT_AS_MILLIS,
                CONNECTION_TIMEOUT_AS_MILLIS,
                SEND_TIMEOUT_AS_MILLIS,
                RECEIVE_TIMEOUT_AS_MILLIS,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        // -------------------- HTTP/2 usage --------------------
        
        let mut version = ffi::WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = mem::size_of::<c_ulong>() as c_ulong;
        
        unsafe {
            
            let result = ffi::WinHttpSetOption(
                handle.as_raw(),
                ffi::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                ptr::from_mut(&mut version).cast::<c_void>(),
                bytes,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(Self {
            handle,
        })
    }
    
}
