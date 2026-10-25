use std::{
    io,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    DNS_RESOLUTION_TIMEOUT, CONNECTION_TIMEOUT,
    SEND_TIMEOUT, RECEIVE_TIMEOUT,
    WinHttpHandle,
};

pub struct Session {
    pub handle: WinHttpHandle,
}

impl Session {
    
    pub fn open(agent: &str) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let wide_agent = chikuwa::win_string(agent)?;
        
        let handle = unsafe {
            
            let result = sys::WinHttpOpen(
                wide_agent.as_ptr(),
                sys::WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        // -------------------- set timeouts --------------------
        
        unsafe {
            
            let result = sys::WinHttpSetTimeouts(
                handle.as_raw(),
                DNS_RESOLUTION_TIMEOUT,
                CONNECTION_TIMEOUT,
                SEND_TIMEOUT,
                RECEIVE_TIMEOUT,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        Ok(Self {
            handle,
        })
    }
    
}
