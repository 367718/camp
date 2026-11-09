use std::{
    os::raw::c_void,
    ptr::NonNull,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    
    // -------------------- winhttperror --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew
    pub fn GetModuleHandleW(
        lp_module_name: *const u16, // LPCWSTR
    ) -> Option<NonNull<c_void>>; // HMODULE
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew
    pub fn FormatMessageW(
        dw_flags: u32, // DWORD
        lp_source: Option<NonNull<c_void>>, // LPCVOID
        dw_message_id: u32, // DWORD
        dw_language_id: u32, // DWORD
        lp_buffer: *mut u16, // LPWSTR
        n_size: u32, // DWORD,
        arguments: *const *const i8, // va_list
    ) -> u32;
    
}

#[link(name = "winhttp")]
unsafe extern "system" {
    
    // -------------------- winhttphandle --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    pub fn WinHttpCloseHandle(
        h_internet: NonNull<c_void>, // HINTERNET
    ) -> i32; // BOOL
    
    // -------------------- url --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpcrackurl
    pub fn WinHttpCrackUrl(
        pwsz_url: *const u16, // LPCWSTR
        dw_url_length: u32, // DWORD
        dw_flags: u32, // DWORD
        lp_url_components: *mut UrlComponents, // LPURL_COMPONENTS
    ) -> i32; // BOOL
    
    // -------------------- session --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen
    pub fn WinHttpOpen(
        psz_agent_w: *const u16, // LPCWSTR
        dw_access_type: u32, // DWORD
        psz_proxy_w: *const u16, // LPCWSTR
        psz_proxy_bypass_w: *const u16, // LPCWSTR
        dw_flags: u32, // DWORD
    ) -> Option<NonNull<c_void>>; // HINTERNET
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts
    pub fn WinHttpSetTimeouts(
        h_internet: NonNull<c_void>, // HINTERNET
        n_resolve_timeout: i32,
        n_connect_timeout: i32,
        n_send_timeout: i32,
        n_receive_timeout: i32,
    ) -> i32; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsetoption
    pub fn WinHttpSetOption(
        h_internet: NonNull<c_void>, // HINTERNET
        dw_option: u32, // DWORD
        lp_buffer: *const c_void, // LPVOID
        dw_buffer_length: u32, // DWORD
    ) -> i32; // BOOL
    
    // -------------------- connection --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    pub fn WinHttpConnect(
        h_session: NonNull<c_void>, // HINTERNET
        pswz_server_name: *const u16, // LPCWSTR
        n_server_port: u16, // INTERNET_PORT
        dw_reserved: u32,
    ) -> Option<NonNull<c_void>>; // HINTERNET
    
    // -------------------- request --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    pub fn WinHttpOpenRequest(
        h_connect: NonNull<c_void>, // HINTERNET
        pwsz_verb: *const u16, // LPCWSTR
        pwsz_object_name: *const u16, // LPCWSTR
        pwsz_version: *const u16, // LPCWSTR
        pwsz_referrer: *const u16, // LPCWSTR
        ppwsz_accept_types: *const *const u16, // LPCWSTR
        dw_flags: u32, // DWORD
    ) -> Option<NonNull<c_void>>; // HINTERNET
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest
    pub fn WinHttpSendRequest(
        h_request: NonNull<c_void>, // HINTERNET
        lpsz_headers: *const u16, // LPCWSTR
        dw_headers_length: u32, // DWORD
        lp_optional: *const c_void, // LPVOID
        dw_optional_length: u32, // DWORD
        dw_total_length: u32, // DWORD
        dw_context: usize, // DWORD_PTR
    ) -> i32; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    pub fn WinHttpReceiveResponse(
        h_request: NonNull<c_void>, // HINTERNET
        lp_reserved: *const c_void, // LPVOID
    ) -> i32; // BOOL
    
    // -------------------- response --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpqueryheaders
    pub fn WinHttpQueryHeaders(
        h_request: NonNull<c_void>, // HINTERNET
        dw_info_level: u32, // DWORD
        pwsz_name: *const u16, // LPCWSTR
        lp_buffer: *mut c_void, // LPVOID
        lpdw_buffer_length: *mut u32, // LPDWORD
        lpdw_index: *mut u32, // LPDWORD
    ) -> i32; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    pub fn WinHttpReadData(
        h_request: NonNull<c_void>, // HINTERNET
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: u32, // DWORD
        lpdw_number_of_bytes_read: *mut u32, // LPDWORD
    ) -> i32; // BOOL
    
}

#[repr(C)]
pub struct UrlComponents {
    pub dw_struct_size: u32, // DWORD
    pub lpsz_scheme: Option<NonNull<u16>>, // LPWSTR
    pub dw_scheme_length: u32, // DWORD
    pub n_scheme: i32, // INTERNET_SCHEME
    pub lpsz_host_name: Option<NonNull<u16>>, // LPWSTR
    pub dw_host_name_length: u32, // DWORD
    pub n_port: u16, // INTERNET_PORT
    pub lpsz_user_name: Option<NonNull<u16>>, // LPWSTR
    pub dw_user_name_length: u32, // DWORD
    pub lpsz_password: Option<NonNull<u16>>, // LPWSTR
    pub dw_password_length: u32, // DWORD
    pub lpsz_url_path: Option<NonNull<u16>>, // LPWSTR
    pub dw_url_path_length: u32, // DWORD
    pub lpsz_extra_info: Option<NonNull<u16>>, // LPWSTR
    pub dw_extra_info_length: u32, // DWORD
}

pub const INTERNET_SCHEME_HTTPS: i32 = 2;

pub const WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY: u32 = 4; // DWORD

pub const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x0000_1000; // DWORD
pub const FORMAT_MESSAGE_FROM_HMODULE: u32 = 0x0000_0800; // DWORD
pub const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x0000_0200; // DWORD

pub const WINHTTP_FLAG_SECURE: u32 = 0x0080_0000; // DWORD

pub const WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL: u32 = 133; // DWORD
pub const WINHTTP_PROTOCOL_FLAG_HTTP2: u32 = 1; // DWORD

pub const WINHTTP_OPTION_REDIRECT_POLICY: u32 = 88; // DWORD
pub const WINHTTP_OPTION_REDIRECT_POLICY_NEVER: u32 = 0; // DWORD

pub const WINHTTP_OPTION_DECOMPRESSION: u32 = 118; // DWORD
pub const WINHTTP_DECOMPRESSION_FLAG_GZIP: u32 = 1; // DWORD
pub const WINHTTP_DECOMPRESSION_FLAG_DEFLATE: u32 = 2; // DWORD

pub const WINHTTP_QUERY_STATUS_CODE: u32 = 19; // DWORD
pub const WINHTTP_QUERY_FLAG_NUMBER: u32 = 0x2000_0000; // DWORD
