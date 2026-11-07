use std::{
    io::{ self, Error, ErrorKind, Read },
    os::raw::*,
    ptr,
};

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Connection,
};

// INVARIANT: drop order required by the WinHTTP API, do not rearrange
// handle -> connection
pub struct Response<'s> {
    handle: WinHttpHandle,
    status_code: u16,
    #[allow(dead_code)]
    connection: Connection<'s>,
}

const READ_BUFFER_MAXIMUM_SIZE: usize = 64 * 1024;

impl<'s> Response<'s> {
    
    pub(crate) fn new(handle: WinHttpHandle, connection: Connection<'s>) -> io::Result<Self> {
        // -------------------- retrieve status_code --------------------
        
        // `WINHTTP_QUERY_STATUS_CODE`: retrieve status code as returned by server
        // `WINHTTP_QUERY_FLAG_NUMBER`: returns the data as a 32-bit number for headers whose value is a number
        let info_level = sys::WINHTTP_QUERY_STATUS_CODE | sys::WINHTTP_QUERY_FLAG_NUMBER;
        
        let mut buffer: c_ulong = 0;
        let mut buffer_length = c_ulong::try_from(size_of_val(&buffer)).expect("Unexpected truncation");
        
        // SAFETY:
        // 1. `handle.as_raw()` is a valid `HINTERNET` request handle on which `WinHttpReceiveResponse` has successfully completed
        // 2. `info_level` is a valid option defined by the WinHTTP API
        // 3. `pwsz_name` as `null` is valid when querying built-in headers selected by `info_level`
        // 4. `(&raw mut buffer).cast::<c_void>()` points to a live, properly aligned `c_ulong` valid for writes
        // 5. `&raw mut buffer_length` points to a live `c_ulong` that matches the size in bytes of `buffer`, preventing out-of-bounds writes
        // 6. `lpdw_index` as `null` is valid and corresponds to the first occurrence of the specified header
        let result = unsafe {
            
            sys::WinHttpQueryHeaders(
                handle.as_raw(),
                info_level,
                ptr::null(), // built-in header
                (&raw mut buffer).cast::<c_void>(),
                &raw mut buffer_length,
                ptr::null_mut(), // retrieve first occurrence
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
        
        let number_of_bytes_to_read = c_ulong::try_from(buf.len().min(READ_BUFFER_MAXIMUM_SIZE)).expect("Unexpected truncation");
        let mut number_of_bytes_read: c_ulong = 0;
        
        // SAFETY:
        // 1. `self.handle.as_raw()` is a valid `HINTERNET` request handle that was verified to be non-null and has not been closed yet
        // 2. `buf.as_mut_ptr().cast::<c_void>()` points to a live, non-empty, caller-provided byte slice valid for writes
        // 3. `number_of_bytes_to_read` cannot exceed `buf` length, preventing out-of-bounds writes
        // 4. `&raw mut number_of_bytes_read` points to a live, properly aligned `c_ulong` valid for writes
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
