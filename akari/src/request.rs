use std::{
    io::{ self, Error },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
    ptr,
};

use crate::{ HttpHandle, Connection, Url };

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    fn WinHttpOpenRequest(
        h_connect: RawHandle,
        pwsz_verb: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_object_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_version: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_referrer: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        ppwsz_accept_types: *mut *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> RawHandle;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest
    fn WinHttpSendRequest(
        h_request: RawHandle,
        lpsz_headers: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_headers_length: c_ulong, // DWORD
        lp_optional: *mut c_void, // LPVOID
        dw_optional_length: c_ulong, // DWORD
        dw_total_length: c_ulong, // DWORD
        dw_context: usize, // DWORD_PTR -> ULONG_PTR
    ) -> c_int; // BOOL
    
}

const WINHTTP_NO_REFERER: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t
const WINHTTP_DEFAULT_ACCEPT_TYPES: *mut *const c_ushort = ptr::null_mut(); // LPCWSTR -> WCHAR -> wchar_t
const WINHTTP_FLAG_SECURE: c_ulong = 0x0080_0000; // DWORD

const WINHTTP_NO_ADDITIONAL_HEADERS: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t

pub struct Request {
    pub handle: HttpHandle,
}

impl Request {
    
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(connection: Connection, url: &Url) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let handle = unsafe {
            
            let result = WinHttpOpenRequest(
                connection.handle.as_raw(),
                ptr::null(),
                chikuwa::win_string(url.path()).as_ptr(),
                ptr::null(),
                WINHTTP_NO_REFERER,
                WINHTTP_DEFAULT_ACCEPT_TYPES,
                WINHTTP_FLAG_SECURE,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            HttpHandle::new(result)
            
        };
        
        // -------------------- send --------------------
        
        unsafe {
            
            let result = WinHttpSendRequest(
                handle.as_raw(),
                WINHTTP_NO_ADDITIONAL_HEADERS,
                0,
                ptr::null_mut(),
                0,
                0,
                0,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        Ok(Self {
            handle,
        })
    }
    
}
