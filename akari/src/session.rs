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
        
        // SAFETY:
        // 1. `wide_agent.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 2. `ptr::null()` is valid for `psz_proxy_w` and `psz_proxy_bypass_w` per the WinHTTP API when not specifying a named proxy
        // 3. `dw_flags` is passed as `0` to signal that no flags will be used
        // 4. on failure, `null` is returned and the proper error is constructed and propagated
        // 5. on success, the returned handle is wrapped in `WinHttpHandle`, which calls `WinHttpCloseHandle` on drop
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
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` session handle
        // 2. timeout values are valid millisecond integer values within the range of `c_int`
        // 3. on failure, `0` is returned and the proper error is constructed and propagated
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
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` session handle
        // 2. `(&raw mut buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong`
        // 3. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        // 4. on failure, `0` is returned and the proper error is constructed and propagated
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
        
        // `WINHTTP_DECOMPRESSION_FLAG_DEFLATE`: decompress "Content-Encoding: gzip" responses
        // `WINHTTP_DECOMPRESSION_FLAG_GZIP`: decompress "Content-Encoding: deflate" responses
        let mut buffer = sys::WINHTTP_DECOMPRESSION_FLAG_GZIP | sys::WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` session handle
        // 2. `(&raw mut buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong`
        // 3. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        // 4. on failure, `0` is returned and the proper error is constructed and propagated
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
