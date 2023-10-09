use std::{
    os::raw::*,
    ptr,
};

extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen
    pub fn WinHttpOpen(
        psz_agent_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_access_type: c_ulong, // DWORD
        psz_proxy_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        psz_proxy_bypass_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> HINTERNET;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts
    pub fn WinHttpSetTimeouts(
        h_internet: HINTERNET,
        n_resolve_timeout: c_int,
        n_connect_timeout: c_int,
        n_send_timeout: c_int,
        n_receive_timeout: c_int,
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    pub fn WinHttpConnect(
        h_session: HINTERNET,
        pswz_server_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        n_server_port: c_ushort, // INTERNET_PORT -> WORD
        dw_reserved: c_ulong,
    ) -> HINTERNET;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    pub fn WinHttpOpenRequest(
        h_connect: HINTERNET,
        pwsz_verb: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_object_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_version: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_referrer: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        ppwsz_accept_types: *mut *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> HINTERNET;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest
    pub fn WinHttpSendRequest(
        h_request: HINTERNET,
        lpsz_headers: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_headers_length: c_ulong, // DWORD
        lp_optional: *mut c_void, // LPVOID
        dw_optional_length: c_ulong, // DWORD
        dw_total_length: c_ulong, // DWORD
        dw_context: usize, // DWORD_PTR -> ULONG_PTR
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    pub fn WinHttpReceiveResponse(
        h_request: HINTERNET,
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpquerydataavailable
    pub fn WinHttpQueryDataAvailable(
        h_request: HINTERNET,
        lpdw_mumber_of_bytes_available: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    pub fn WinHttpReadData(
        h_request: HINTERNET,
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    pub fn WinHttpCloseHandle(
        h_internet: HINTERNET,
    ) -> c_int; // BOOL
    
}

#[allow(clippy::upper_case_acronyms)]
pub type HINTERNET = *mut c_void;

pub const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: c_ulong = 0; // DWORD
pub const WINHTTP_NO_PROXY_NAME: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t
pub const WINHTTP_NO_PROXY_BYPASS: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t

pub const WINHTTP_NO_REFERER: *const c_ushort = ptr::null(); // // LPCWSTR -> WCHAR -> wchar_t
pub const WINHTTP_DEFAULT_ACCEPT_TYPES: *mut *const c_ushort = ptr::null_mut(); // // LPCWSTR -> WCHAR -> wchar_t
pub const WINHTTP_FLAG_SECURE: c_ulong = 0x0080_0000; // DWORD

pub const WINHTTP_NO_ADDITIONAL_HEADERS: *const c_ushort = ptr::null(); // // LPCWSTR -> WCHAR -> wchar_t
pub const WINHTTP_NO_REQUEST_DATA: *mut c_void = ptr::null_mut(); // LPVOID
