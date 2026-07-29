use std::{
    io::{ self, Error, ErrorKind, Read },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
    ptr,
};

use super::HttpHandle;

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpqueryheaders
    fn WinHttpQueryHeaders(
        h_request: RawHandle,
        dw_info_level: c_ulong, // DWORD
        pwsz_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        lp_buffer: *mut c_void, // LPVOID
        lpdw_buffer_length: *mut c_ulong, // LPDWORD -> DWORD
        lpdw_index: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    fn WinHttpReadData(
        h_request: RawHandle,
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
}

const WINHTTP_QUERY_CONTENT_LENGTH: c_ulong = 5; // DWORD
const WINHTTP_QUERY_FLAG_NUMBER: c_ulong = 0x2000_0000; // DWORD

pub struct Response {
    pub(crate) handle: HttpHandle,
}

impl Response {
    
    pub fn content_length(&self) -> io::Result<u64> {
        let mut content_length: c_ulong = 0;
        
        #[allow(clippy::cast_possible_truncation)]
        let mut bytes = size_of_val(&content_length) as c_ulong;
        
        unsafe {
            
            let result = WinHttpQueryHeaders(
                self.handle.as_raw(),
                WINHTTP_QUERY_CONTENT_LENGTH | WINHTTP_QUERY_FLAG_NUMBER,
                ptr::null(),
                ptr::from_mut(&mut content_length).cast::<c_void>(),
                &raw mut bytes,
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(u64::from(content_length))
    }
    
}

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: c_ulong = 0;
        
        let bytes = c_ulong::try_from(buf.len())
            .map_err(|_| Error::from(ErrorKind::InvalidInput))?;
        
        unsafe {
            
            let result = WinHttpReadData(
                self.handle.as_raw(),
                ptr::from_mut(buf).cast::<c_void>(),
                bytes,
                &raw mut amount_read,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(amount_read as usize)
    }
    
}
