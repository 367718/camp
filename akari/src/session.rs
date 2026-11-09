use std::{
    io,
    os::raw::c_void,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle,
};

pub(crate) struct Session {
    pub(crate) handle: WinHttpHandle,
}

const DNS_RESOLUTION_TIMEOUT: i32 = 15_000;
const CONNECTION_TIMEOUT: i32 = 15_000;
const SEND_TIMEOUT: i32 = 15_000;
const RECEIVE_TIMEOUT: i32 = 15_000;

impl Session {
    
    pub(crate) fn open(agent: &str) -> io::Result<Self> {
        let agent_wide = chikuwa::win_string(agent)?;
        
        // -------------------- open --------------------
        
        // uses system proxy settings
        // available starting with Windows 8.1
        let access_type = sys::WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY;
        
        // SAFETY:
        // 1. `agent_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 2. `access_type` is a valid option defined by the WinHTTP API
        // 3. `null` is valid for `psz_proxy_w` and `psz_proxy_bypass_w` per the WinHTTP API when not specifying a named proxy
        // 4. `dw_flags` as `0` is valid and selects standard WinHTTP API defaults
        let result = unsafe {
            
            sys::WinHttpOpen(
                agent_wide.as_ptr(),
                access_type,
                ptr::null(), // no proxy name
                ptr::null(), // no proxy bypass
                0, // no async, no secure defaults
            )
            
        };
        
        let handle = result
            // SAFETY: `raw` is a uniquely owned, valid, open session handle
            .map(|raw| unsafe { WinHttpHandle::from_raw(raw) })
            .ok_or_else(last_winhttp_error)?;
        
        // -------------------- set timeouts --------------------
        
        // SAFETY: `handle.inner()` is a valid, open `HINTERNET` session handle (guaranteed non-null by `WinHttpHandle`)
        let result = unsafe {
            
            sys::WinHttpSetTimeouts(
                handle.inner(),
                DNS_RESOLUTION_TIMEOUT,
                CONNECTION_TIMEOUT,
                SEND_TIMEOUT,
                RECEIVE_TIMEOUT,
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- set HTTP/2 --------------------
        
        // allowed HTTP protocol versions
        // available starting with Windows 10 version 1607
        let option = sys::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL;
        
        // enables HTTP/2 for all requests
        let buffer = sys::WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        // SAFETY:
        // 1. `handle.inner()` is a valid, open `HINTERNET` session handle (guaranteed non-null by `WinHttpHandle`)
        // 2. `option` is a valid option defined by the WinHTTP API
        // 3. `(&raw const buffer).cast::<c_void>()` points to a live, properly aligned `u32`
        // 4. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        let result = unsafe {
            
            sys::WinHttpSetOption(
                handle.inner(),
                option,
                (&raw const buffer).cast::<c_void>(),
                u32::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- set redirect policy --------------------
        
        // controls whether to follow redirections automatically
        let option = sys::WINHTTP_OPTION_REDIRECT_POLICY;
        
        // disable automatic following of redirections
        let buffer = sys::WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
        
        // SAFETY:
        // 1. `handle.inner()` is a valid, open `HINTERNET` session handle (guaranteed non-null by `WinHttpHandle`)
        // 2. `option` is a valid option defined by the WinHTTP API
        // 3. `(&raw const buffer).cast::<c_void>()` points to a live, properly aligned `u32`
        // 4. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        let result = unsafe {
            
            sys::WinHttpSetOption(
                handle.inner(),
                option,
                (&raw const buffer).cast::<c_void>(),
                u32::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- set decompression --------------------
        
        // automatically decompress response bodies with compressed "Content-Encoding"
        // this will also set an appropriate "Accept-Encoding" header for all requests unless one has already been set manually
        // available on Windows 8.1 and later
        let option = sys::WINHTTP_OPTION_DECOMPRESSION;
        
        // `WINHTTP_DECOMPRESSION_FLAG_DEFLATE`: decompress "Content-Encoding: deflate" responses
        // `WINHTTP_DECOMPRESSION_FLAG_GZIP`: decompress "Content-Encoding: gzip" responses
        let buffer = sys::WINHTTP_DECOMPRESSION_FLAG_GZIP | sys::WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        
        // SAFETY:
        // 1. `handle.inner()` is a valid, open `HINTERNET` session handle (guaranteed non-null by `WinHttpHandle`)
        // 2. `option` is a valid option defined by the WinHTTP API
        // 3. `(&raw const buffer).cast::<c_void>()` points to a live, properly aligned `u32`
        // 4. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        let result = unsafe {
            
            sys::WinHttpSetOption(
                handle.inner(),
                option,
                (&raw const buffer).cast::<c_void>(),
                u32::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        Ok(Self { handle })
    }
    
}
