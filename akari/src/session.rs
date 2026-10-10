use std::{
    io::{ self, Error },
    os::{
        raw::*,
        windows::io::{ RawHandle, AsRawHandle },
    },
    ptr,
};

use crate::{
    DNS_RESOLUTION_TIMEOUT,
    CONNECTION_TIMEOUT,
    SEND_TIMEOUT,
    RECEIVE_TIMEOUT,
    WinHttpHandle,
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
const WINHTTP_OPTION_DECOMPRESSION: c_ulong = 118; // DWORD
const WINHTTP_DECOMPRESSION_FLAG_GZIP: c_ulong = 1; // DWORD
const WINHTTP_DECOMPRESSION_FLAG_DEFLATE: c_ulong = 2; // DWORD

pub struct Session {
    pub handle: WinHttpHandle,
}

impl Session {
    
    pub fn open(agent: &str) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let handle = unsafe {
            
            let result = WinHttpOpen(
                chikuwa::win_string(agent)?.as_ptr(),
                WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            WinHttpHandle::from(result)
            
        };
        
        // -------------------- timeouts --------------------
        
        unsafe {
            
            let result = WinHttpSetTimeouts(
                handle.as_raw_handle(),
                DNS_RESOLUTION_TIMEOUT,
                CONNECTION_TIMEOUT,
                SEND_TIMEOUT,
                RECEIVE_TIMEOUT,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        // -------------------- http2 --------------------
        
        let mut flag = WINHTTP_PROTOCOL_FLAG_HTTP2;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&flag) as c_ulong;
        
        unsafe {
            
            let result = WinHttpSetOption(
                handle.as_raw_handle(),
                WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL,
                ptr::from_mut(&mut flag).cast::<c_void>(),
                bytes,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        // -------------------- decompression --------------------
        
        let mut flag = WINHTTP_DECOMPRESSION_FLAG_GZIP | WINHTTP_DECOMPRESSION_FLAG_DEFLATE;
        
        #[allow(clippy::cast_possible_truncation)]
        let bytes = size_of_val(&flag) as c_ulong;
        
        unsafe {
            
            let result = WinHttpSetOption(
                handle.as_raw_handle(),
                WINHTTP_OPTION_DECOMPRESSION,
                ptr::from_mut(&mut flag).cast::<c_void>(),
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
