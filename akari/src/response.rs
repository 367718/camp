use std::{
    io::{ self, Error, ErrorKind, Read },
    os::raw::*,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Connection,
};

// SAFETY: drop order required by winhttp, do not rearrange
pub struct Response<'s> {
    handle: WinHttpHandle,
    status_code: u16,
    #[allow(dead_code)]
    connection: Connection<'s>,
}

impl<'s> Response<'s> {
    
    pub(crate) fn receive(connection: Connection<'s>, path: &str) -> io::Result<Self> {
        // -------------------- open request --------------------
        
        let path_wide = chikuwa::win_string(path)?;
        
        // enables TLS
        let flags = sys::WINHTTP_FLAG_SECURE;
        
        // SAFETY:
        // 1. `connection.handle.as_raw()` is a valid `HINTERNET` connection handle whose lifetime is guaranteed by the owned `Connection`
        // 2. `path_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. passing `null` for `pwsz_verb`, `pwsz_version`, `pwsz_referrer`, and `ppwsz_accept_types` selects standard WinHTTP defaults
        let result = unsafe {
            
            sys::WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(), // default http verb: GET
                path_wide.as_ptr(),
                ptr::null(), // default and (highest supported) http version: HTTP/1.1
                ptr::null(), // no referer
                ptr::null_mut(), // no accept header
                flags,
            )
            
        };
        
        // SAFETY: if non-null, `result` is a valid, open request handle uniquely owned
        let handle = unsafe { WinHttpHandle::from_raw(result)? };
        
        // -------------------- send request --------------------
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` request handle that was verified to be non-null and has not been closed yet
        // 2. `null` for `lpsz_headers` and `0` for `dw_headers_length` are valid if no additional headers are to be sent
        // 3. `null` for `lp_optional` and `0` for `dw_optional_length` are valid if no body is to be sent
        // 4. `dw_total_length` as `0` is valid as no body is to be sent
        // 5. `dw_context` as `0` is valid and signals that no context is associated with the request
        let result = unsafe {
            
            sys::WinHttpSendRequest(
                handle.as_raw(),
                ptr::null(), // no additional headers
                0, // no additional headers
                ptr::null_mut(), // no body
                0, // no body
                0, // no body
                0, // no context
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- receive response --------------------
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` request handle on which `WinHttpSendRequest` has successfully completed
        // 2. `lp_reserved` as `ptr::null_mut()` is required by the WinHTTP API
        let result = unsafe {
            
            sys::WinHttpReceiveResponse(
                handle.as_raw(),
                ptr::null_mut(), // reserved
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        // -------------------- retrieve status_code --------------------
        
        // `WINHTTP_QUERY_STATUS_CODE`: retrieve status code as returned by server
        // `WINHTTP_QUERY_FLAG_NUMBER`: returns the data as a 32-bit number for headers whose value is a number
        let info_level = sys::WINHTTP_QUERY_STATUS_CODE | sys::WINHTTP_QUERY_FLAG_NUMBER;
        
        let mut buffer: c_ulong = 0;
        let mut buffer_length = c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation");
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` request handle on which `WinHttpReceiveResponse` has successfully completed
        // 2. `pwsz_name` as `null` is valid when querying built-in headers selected by `info_level`
        // 3. `(&raw mut buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong`
        // 4. `&raw mut buffer_length` points to a live `c_ulong` that matches the size in bytes of `buffer`, preventing out-of-bounds writes
        // 5. `lpdw_index` as `null` is valid and corresponds to the first occurrence of the specified header
        let result = unsafe {
            
            sys::WinHttpQueryHeaders(
                handle.as_raw(),
                info_level,
                ptr::null(), // built-in header
                (&raw mut buffer).cast::<c_void>(),
                &raw mut buffer_length,
                ptr::null_mut(), // single-value header
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        let status_code = u16::try_from(buffer)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid HTTP response status code"))?;
        
        // `connection` is kept alive to prevent its `handle` getting closed
        Ok(Self { handle, status_code, connection })
    }
    
    pub fn status_code(&self) -> u16 {
        self.status_code
    }
    
}

impl Read for Response<'_> {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        
        let number_of_bytes_to_read = c_ulong::try_from(buf.len().min(c_ulong::MAX as usize)).expect("Unexpected truncation");
        let mut number_of_bytes_read: c_ulong = 0;
        
        // SAFETY:
        // 1. `self.handle.as_raw()` is a valid `HINTERNET` request handle that was verified to be non-null and has not been closed yet
        // 2. `buf.as_mut_ptr().cast::<c_void>()` points to a live, non-empty, caller-provided byte slice valid for writes
        // 3. `number_of_bytes_to_read` cannot exceed `buf` length, preventing out-of-bounds writes
        // 4. `&raw mut number_of_bytes_read` points to a live, properly aligned `c_ulong`
        let result = unsafe {
            
            sys::WinHttpReadData(
                self.handle.as_raw(),
                buf.as_mut_ptr().cast::<c_void>(),
                number_of_bytes_to_read,
                &raw mut number_of_bytes_read,
            )
            
        };
        
        if result == 0 {
            return Err(last_winhttp_error());
        }
        
        Ok(number_of_bytes_read as usize)
    }
    
}
