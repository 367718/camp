use std::{
    io::{ self, Error, ErrorKind, Read },
    os::raw::*,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Connection,
};

pub struct Response {
    handle: WinHttpHandle,
}

impl Response {
    
    pub(crate) fn receive(connection: &Connection, path: &str) -> io::Result<Self> {
        // -------------------- open request --------------------
        
        let wide_path = chikuwa::win_string(path)?;
        
        let handle = unsafe {
            
            let result = sys::WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(),
                wide_path.as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                sys::WINHTTP_FLAG_SECURE,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        // -------------------- set http2 --------------------
        
        let mut flag = sys::WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&flag) as c_ulong;
        
        unsafe {
            
            let result = sys::WinHttpSetOption(
                handle.as_raw(),
                sys::WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                ptr::from_mut(&mut flag).cast::<c_void>(),
                bytes,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- set decompression --------------------
        
        // winhttp does not natively support the brotli (br) compression algorithm
        let mut flag = sys::WINHTTP_DECOMPRESSION_FLAG_GZIP | sys::WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&flag) as c_ulong;
        
        unsafe {
            
            let result = sys::WinHttpSetOption(
                handle.as_raw(),
                sys::WINHTTP_OPTION_DECOMPRESSION,
                ptr::from_mut(&mut flag).cast::<c_void>(),
                bytes,
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
        
        Ok(Self {
            handle,
        })
    }
    
}

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: c_ulong = 0;
        
        let bytes = c_ulong::try_from(buf.len())
            .map_err(|_| Error::from(ErrorKind::InvalidInput))?;
        
        unsafe {
            
            let result = sys::WinHttpReadData(
                self.handle.as_raw(),
                ptr::from_mut(buf).cast::<c_void>(),
                bytes,
                &raw mut amount_read,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        Ok(amount_read as usize)
    }
    
}
