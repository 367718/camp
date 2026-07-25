use std::{
    io::{ self, Error, Read },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
    ptr,
};

use super::{ HttpHandle, Request };

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    fn WinHttpReceiveResponse(
        h_request: RawHandle,
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
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
    handle: HttpHandle,
}

impl Response {
    
    pub fn new(request: Request) -> io::Result<Self> {
        unsafe {
            
            let result = WinHttpReceiveResponse(
                request.handle.as_raw(),
                ptr::null_mut(),
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(Self {
            handle: request.handle,
        })
    }
    
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
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = buf.len() as c_ulong;
        
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
