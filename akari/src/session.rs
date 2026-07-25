use std::{
    io::{ self, Error },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
    ptr,
};

use crate::{
    HttpHandle,
    DNS_RESOLUTION_TIMEOUT_AS_MILLIS, CONNECTION_TIMEOUT_AS_MILLIS,
    SEND_TIMEOUT_AS_MILLIS, RECEIVE_TIMEOUT_AS_MILLIS,
};

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen
    fn WinHttpOpen(
        psz_agent_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_access_type: c_ulong, // DWORD
        psz_proxy_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        psz_proxy_bypass_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> RawHandle;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts
    fn WinHttpSetTimeouts(
        h_internet: RawHandle,
        n_resolve_timeout: c_int,
        n_connect_timeout: c_int,
        n_send_timeout: c_int,
        n_receive_timeout: c_int,
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsetoption
    fn WinHttpSetOption(
        h_internet: RawHandle,
        dw_option: c_ulong, // DWORD
        lp_buffer: *mut c_void, // LPVOID
        dw_buffer_length: c_ulong, // DWORD
    ) -> c_int; // BOOL
    
}

const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: c_ulong = 0; // DWORD
const WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL: c_ulong = 133; // DWORD
const WINHTTP_PROTOCOL_FLAG_HTTP2: c_ulong = 1; // DWORD

pub struct Session {
    pub handle: HttpHandle,
}

impl Session {
    
    pub fn new(agent: &str) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let handle = unsafe {
            
            let result = WinHttpOpen(
                chikuwa::win_string(agent).as_ptr(),
                WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            HttpHandle::new(result)
            
        };
        
        // -------------------- timeouts --------------------
        
        unsafe {
            
            let result = WinHttpSetTimeouts(
                handle.as_raw(),
                DNS_RESOLUTION_TIMEOUT_AS_MILLIS,
                CONNECTION_TIMEOUT_AS_MILLIS,
                SEND_TIMEOUT_AS_MILLIS,
                RECEIVE_TIMEOUT_AS_MILLIS,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        // -------------------- http2 --------------------
        
        let mut version = WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&version) as c_ulong;
        
        unsafe {
            
            let result = WinHttpSetOption(
                handle.as_raw(),
                WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                ptr::from_mut(&mut version).cast::<c_void>(),
                bytes,
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
