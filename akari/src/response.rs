use std::{
    io::{ self, Error, ErrorKind, Read },
    os::raw::*,
    ptr,
};

use crate::{
    last_winhttp_error,
    WinHttpHandle,
    Connection,
};

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    fn WinHttpOpenRequest(
        h_connect: *mut c_void, // HINTERNET -> LPVOID
        pwsz_verb: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_object_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_version: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_referrer: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        ppwsz_accept_types: *mut *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> *mut c_void; // HINTERNET -> LPVOID
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsetoption
    fn WinHttpSetOption(
        h_internet: *mut c_void, // HINTERNET -> LPVOID
        dw_option: c_ulong, // DWORD
        lp_buffer: *mut c_void, // LPVOID
        dw_buffer_length: c_ulong, // DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest
    fn WinHttpSendRequest(
        h_request: *mut c_void, // HINTERNET -> LPVOID
        lpsz_headers: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_headers_length: c_ulong, // DWORD
        lp_optional: *mut c_void, // LPVOID
        dw_optional_length: c_ulong, // DWORD
        dw_total_length: c_ulong, // DWORD
        dw_context: usize, // DWORD_PTR -> ULONG_PTR
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    fn WinHttpReceiveResponse(
        h_request: *mut c_void, // HINTERNET -> LPVOID
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    fn WinHttpReadData(
        h_request: *mut c_void, // HINTERNET -> LPVOID
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
}

const WINHTTP_FLAG_SECURE: c_ulong = 0x0080_0000; // DWORD

const WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL: c_ulong = 133; // DWORD
const WINHTTP_PROTOCOL_FLAG_HTTP2: c_ulong = 1; // DWORD
const WINHTTP_OPTION_DECOMPRESSION: c_ulong = 118; // DWORD
const WINHTTP_DECOMPRESSION_FLAG_GZIP: c_ulong = 1; // DWORD
const WINHTTP_DECOMPRESSION_FLAG_DEFLATE: c_ulong = 2; // DWORD

pub struct Response {
    handle: WinHttpHandle,
}

impl Response {
    
    pub(crate) fn receive(connection: &Connection, path: &str) -> io::Result<Self> {
        // -------------------- open request --------------------
        
        let handle = unsafe {
            
            let result = WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(),
                chikuwa::win_string(path)?.as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                WINHTTP_FLAG_SECURE,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        // -------------------- set http2 --------------------
        
        let mut flag = WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&flag) as c_ulong;
        
        unsafe {
            
            let result = WinHttpSetOption(
                handle.as_raw(),
                WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                ptr::from_mut(&mut flag).cast::<c_void>(),
                bytes,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- set decompression --------------------
        
        // winhttp does not natively support the brotli (br) compression algorithm
        let mut flag = WINHTTP_DECOMPRESSION_FLAG_GZIP | WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&flag) as c_ulong;
        
        unsafe {
            
            let result = WinHttpSetOption(
                handle.as_raw(),
                WINHTTP_OPTION_DECOMPRESSION,
                ptr::from_mut(&mut flag).cast::<c_void>(),
                bytes,
            );
            
            if result == 0 {
                return Err(last_winhttp_error());
            }
            
        }
        
        // -------------------- send request --------------------
        
        unsafe {
            
            let result = WinHttpSendRequest(
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
            
            let result = WinHttpReceiveResponse(
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
            
            let result = WinHttpReadData(
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
