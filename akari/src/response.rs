use std::{
    io::{ self, Error, Read },
    os::raw::*,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Connection,
};

pub struct Response {
    handle: WinHttpHandle,
    #[allow(dead_code)]
    connection: Connection,
}

impl Response {
    
    pub(crate) fn receive(connection: Connection, path: &str) -> io::Result<Self> {
        // -------------------- open request --------------------
        
        let wide_path = chikuwa::win_string(path)?;
        
        // enables TLS
        let flags = sys::WINHTTP_FLAG_SECURE;
        
        let handle = unsafe {
            
            let result = sys::WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(), // default http verb: GET
                wide_path.as_ptr(),
                ptr::null(), // default and (highest supported) http version: HTTP/1.1
                ptr::null(), // no referer
                ptr::null_mut(), // no accept header
                flags,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        // -------------------- send request --------------------
        
        unsafe {
            
            let result = sys::WinHttpSendRequest(
                handle.as_raw(),
                ptr::null(), // no additional headers
                0, // no additional headers
                ptr::null_mut(), // no optional data
                0, // no optional data
                0, // request does not have a body
                0, // no context
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- receive response --------------------
        
        unsafe {
            
            let result = sys::WinHttpReceiveResponse(
                handle.as_raw(),
                ptr::null_mut(), // reserved
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- check status code --------------------
        
        // WINHTTP_QUERY_STATUS_CODE: retrieve status code as returned by server
        // WINHTTP_QUERY_FLAG_NUMBER: returns the data as a 32-bit number for headers whose value is a number
        let info_level = sys::WINHTTP_QUERY_STATUS_CODE | sys::WINHTTP_QUERY_FLAG_NUMBER;
        
        let mut buffer: c_ulong = 0;
        let mut buffer_length = c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation");
        
        unsafe {
            
            let result = sys::WinHttpQueryHeaders(
                handle.as_raw(),
                info_level,
                ptr::null(), // built-in header
                (&raw mut buffer).cast::<c_void>(),
                &raw mut buffer_length,
                ptr::null_mut(), // single-value header
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        if buffer != 200 {
            return Err(Error::other(format!("Server responded with status code {}", buffer)));
        }
        
        // "connection" is kept alive to prevent triggering its "drop" implementation
        Ok(Self { handle, connection })
    }
    
}

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        
        let to_read = c_ulong::try_from(buf.len().min(c_ulong::MAX as usize)).expect("Unexpected truncation");
        let mut amount_read: c_ulong = 0;
        
        unsafe {
            
            let result = sys::WinHttpReadData(
                self.handle.as_raw(),
                buf.as_mut_ptr().cast::<c_void>(),
                to_read,
                &raw mut amount_read,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        Ok(amount_read as usize)
    }
    
}
