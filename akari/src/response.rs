use std::{
    io::{ self, Read },
    os::raw::*,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Connection,
};

pub struct Response {
    handle: WinHttpHandle,
    status_code: u16,
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
                ptr::null(),
                wide_path.as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                flags,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        // -------------------- set http2 --------------------
        
        // acceptable HTTP version
        let option = sys::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL;
        
        // enables HTTP/2 for the request
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
        // this will also set an appropriate "Accept-Encoding" header
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
        
        // -------------------- send request --------------------
        
        unsafe {
            
            let result = sys::WinHttpSendRequest(
                handle.as_raw(),
                ptr::null(),
                0,
                ptr::null_mut(),
                0,
                0,
                0,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- receive response --------------------
        
        unsafe {
            
            let result = sys::WinHttpReceiveResponse(
                handle.as_raw(),
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- query status code --------------------
        
        // WINHTTP_QUERY_STATUS_CODE: retrieve status code as returned by server
        // WINHTTP_QUERY_FLAG_NUMBER: returns the data as a 32-bit number for headers whose value is a number
        let info_level = sys::WINHTTP_QUERY_STATUS_CODE | sys::WINHTTP_QUERY_FLAG_NUMBER;
        
        let mut buffer: c_ulong = 0;
        let mut buffer_length = c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation");
        
        unsafe {
            
            let result = sys::WinHttpQueryHeaders(
                handle.as_raw(),
                info_level,
                ptr::null(),
                (&raw mut buffer).cast::<c_void>(),
                &raw mut buffer_length,
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        let status_code = u16::try_from(buffer).expect("Unexpected truncation");
        
        Ok(Self { handle, status_code, connection })
    }
    
    pub fn status_code(&self) -> u16 {
        self.status_code
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
