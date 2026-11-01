use std::{
    io,
    os::raw::*,
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
        
        // uses system proxy settings
        let access_type = sys::WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY;
        
        let handle = unsafe {
            
            let result = sys::WinHttpOpen(
                wide_agent.as_ptr(),
                access_type,
                ptr::null(), // no proxy name
                ptr::null(), // no proxy bypass
                0, // no async, no secure defaults
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
        
        // -------------------- set HTTP/2 --------------------
        
        // acceptable HTTP version
        // available on Windows 10 and later
        let option = sys::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL;
        
        // enables HTTP/2 for all requests
        let mut buffer = sys::WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        unsafe {
            
            let result = sys::WinHttpSetOption(
                handle.as_raw(),
                option,
                (&raw mut buffer).cast::<c_void>(),
                c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- set decompression --------------------
        
        // automatically decompress response bodies with compressed "Content-Encoding"
        // this will also set an appropriate "Accept-Encoding" header for all requests
        // available on Windows 8.1 and later
        let option = sys::WINHTTP_OPTION_DECOMPRESSION;
        
        // WINHTTP_DECOMPRESSION_FLAG_DEFLATE: decompress "Content-Encoding: gzip" responses
        // WINHTTP_DECOMPRESSION_FLAG_GZIP: decompress "Content-Encoding: deflate" responses
        let mut buffer = sys::WINHTTP_DECOMPRESSION_FLAG_GZIP | sys::WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        
        unsafe {
            
            let result = sys::WinHttpSetOption(
                handle.as_raw(),
                option,
                (&raw mut buffer).cast::<c_void>(),
                c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        Ok(Self { handle })
    }
    
}
