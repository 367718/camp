use std::{
    io,
    os::raw::*,
    ptr::{ self, NonNull },
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle,
};

pub(crate) struct Session {
    pub(crate) handle: WinHttpHandle,
}

const DNS_RESOLUTION_TIMEOUT: c_int = 15_000;
const CONNECTION_TIMEOUT: c_int = 15_000;
const SEND_TIMEOUT: c_int = 15_000;
const RECEIVE_TIMEOUT: c_int = 15_000;

impl Session {
    
    pub(crate) fn open(agent: &str) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let agent_wide = chikuwa::win_string(agent)?;
        
        // uses system proxy settings
        let access_type = sys::WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY;
        
        // SAFETY:
        // 1. `agent_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 2. `access_type` as `WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY` is valid starting with Windows 8.1
        // 3. `ptr::null()` is valid for `psz_proxy_w` and `psz_proxy_bypass_w` per the WinHTTP API when not specifying a named proxy
        // 4. `dw_flags` is passed as `0` to signal that no flags will be used
        let result = unsafe {
            
            sys::WinHttpOpen(
                agent_wide.as_ptr(),
                access_type,
                ptr::null(), // no proxy name
                ptr::null(), // no proxy bypass
                0, // no async, no secure defaults
            )
            
        };
        
        let handle = match NonNull::new(result) {
            // SAFETY: `result` is a uniquely owned, valid, open session handle
            Some(result) => unsafe { WinHttpHandle::from_raw(result) },
            None => return Err(last_winhttp_error()),
        };
        
        // -------------------- set timeouts --------------------
        
        // SAFETY: `handle.as_raw()` is a valid `HINTERNET` session handle that was verified to be non-null and has not been closed yet
        let result = unsafe {
            
            sys::WinHttpSetTimeouts(
                handle.as_raw(),
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
        let option = sys::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL;
        
        // enables HTTP/2 for all requests
        let buffer = sys::WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` session handle that was verified to be non-null and has not been closed yet
        // 2. `option` as `WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL` is valid starting with Windows 10 version 1607
        // 3. `(&raw const buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong`
        // 4. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        // 5. `WinHttpSetOption` only reads from `lp_buffer`, casting to `*mut c_void` does not mutate immutable memory
        let result = unsafe {
            
            sys::WinHttpSetOption(
                handle.as_raw(),
                option,
                (&raw const buffer).cast::<c_void>().cast_mut(),
                c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
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
        // 1. `handle.as_raw()` is a valid `HINTERNET` session handle that was verified to be non-null and has not been closed yet
        // 2. `(&raw const buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong`
        // 3. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        // 4. `WinHttpSetOption` only reads from `lp_buffer`, casting to `*mut c_void` does not mutate immutable memory
        let result = unsafe {
            
            sys::WinHttpSetOption(
                handle.as_raw(),
                option,
                (&raw const buffer).cast::<c_void>().cast_mut(),
                c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
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
        // 1. `handle.as_raw()` is a valid `HINTERNET` session handle that was verified to be non-null and has not been closed yet
        // 2. `(&raw const buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong`
        // 3. `size_of_val(&buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds reads
        // 4. `WinHttpSetOption` only reads from `lp_buffer`, casting to `*mut c_void` does not mutate immutable memory
        let result = unsafe {
            
            sys::WinHttpSetOption(
                handle.as_raw(),
                option,
                (&raw const buffer).cast::<c_void>().cast_mut(),
                c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation"),
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        Ok(Self { handle })
    }
    
}
