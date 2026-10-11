use std::{
    io,
    os::raw::*,
    ptr,
};

use crate::{
    DNS_RESOLUTION_TIMEOUT,
    CONNECTION_TIMEOUT,
    SEND_TIMEOUT,
    RECEIVE_TIMEOUT,
    last_winhttp_error,
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
    ) -> *mut c_void; // HINTERNET -> LPVOID
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts
    fn WinHttpSetTimeouts(
        h_internet: *mut c_void, // HINTERNET -> LPVOID
        n_resolve_timeout: c_int,
        n_connect_timeout: c_int,
        n_send_timeout: c_int,
        n_receive_timeout: c_int,
    ) -> c_int; // BOOL
    
}

const WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY: c_ulong = 4; // DWORD

pub struct Session {
    pub handle: WinHttpHandle,
}

impl Session {
    
    pub fn open(agent: &str) -> io::Result<Self> {
        // -------------------- open --------------------
        
        let handle = unsafe {
            
            let result = WinHttpOpen(
                chikuwa::win_string(agent)?.as_ptr(),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                ptr::null(),
                ptr::null(),
                0,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        // -------------------- timeouts --------------------
        
        unsafe {
            
            let result = WinHttpSetTimeouts(
                handle.as_raw(),
                DNS_RESOLUTION_TIMEOUT,
                CONNECTION_TIMEOUT,
                SEND_TIMEOUT,
                RECEIVE_TIMEOUT,
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
