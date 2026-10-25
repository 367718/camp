use std::os::raw::*;

unsafe extern "system" {
    
    // -------------------- winhttperror --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew
    pub fn GetModuleHandleW(
        lp_module_name: *const c_ushort, // LPCWSTR
    ) -> *mut c_void; // HMODULE
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew
    pub fn FormatMessageW(
        dw_flags: c_ulong, // DWORD
        lp_source: *const c_void, // LPCVOID
        dw_message_id: c_ulong, // DWORD
        dw_language_id: c_ulong, // DWORD
        lp_buffer: *mut c_ushort, // LPWSTR
        n_size: c_ulong, // DWORD,
        arguments: *mut c_char, // va_list
    ) -> c_ulong;
    
    // -------------------- winhttphandle --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    pub fn WinHttpCloseHandle(
        h_internet: *mut c_void, // HINTERNET
    ) -> c_int; // BOOL
    
    // -------------------- session --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen
    pub fn WinHttpOpen(
        psz_agent_w: *const c_ushort, // LPCWSTR
        dw_access_type: c_ulong, // DWORD
        psz_proxy_w: *const c_ushort, // LPCWSTR
        psz_proxy_bypass_w: *const c_ushort, // LPCWSTR
        dw_flags: c_ulong, // DWORD
    ) -> *mut c_void; // HINTERNET
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts
    pub fn WinHttpSetTimeouts(
        h_internet: *mut c_void, // HINTERNET
        n_resolve_timeout: c_int,
        n_connect_timeout: c_int,
        n_send_timeout: c_int,
        n_receive_timeout: c_int,
    ) -> c_int; // BOOL
    
    // -------------------- connection --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    pub fn WinHttpConnect(
        h_session: *mut c_void, // HINTERNET
        pswz_server_name: *const c_ushort, // LPCWSTR
        n_server_port: c_ushort, // INTERNET_PORT
        dw_reserved: c_ulong,
    ) -> *mut c_void; // HINTERNET
    
    // -------------------- response --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    pub fn WinHttpOpenRequest(
        h_connect: *mut c_void, // HINTERNET
        pwsz_verb: *const c_ushort, // LPCWSTR
        pwsz_object_name: *const c_ushort, // LPCWSTR
        pwsz_version: *const c_ushort, // LPCWSTR
        pwsz_referrer: *const c_ushort, // LPCWSTR
        ppwsz_accept_types: *mut *const c_ushort, // LPCWSTR
        dw_flags: c_ulong, // DWORD
    ) -> *mut c_void; // HINTERNET
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsetoption
    pub fn WinHttpSetOption(
        h_internet: *mut c_void, // HINTERNET
        dw_option: c_ulong, // DWORD
        lp_buffer: *mut c_void, // LPVOID
        dw_buffer_length: c_ulong, // DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest
    pub fn WinHttpSendRequest(
        h_request: *mut c_void, // HINTERNET
        lpsz_headers: *const c_ushort, // LPCWSTR
        dw_headers_length: c_ulong, // DWORD
        lp_optional: *mut c_void, // LPVOID
        dw_optional_length: c_ulong, // DWORD
        dw_total_length: c_ulong, // DWORD
        dw_context: usize, // DWORD_PTR
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    pub fn WinHttpReceiveResponse(
        h_request: *mut c_void, // HINTERNET
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    pub fn WinHttpReadData(
        h_request: *mut c_void, // HINTERNET
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD
    ) -> c_int; // BOOL
    
}

pub const WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY: c_ulong = 4; // DWORD

pub const FORMAT_MESSAGE_FROM_SYSTEM: c_ulong = 0x0000_1000;
pub const FORMAT_MESSAGE_FROM_HMODULE: c_ulong = 0x0000_0800;
pub const FORMAT_MESSAGE_IGNORE_INSERTS: c_ulong = 0x0000_0200;

pub const WINHTTP_FLAG_SECURE: c_ulong = 0x0080_0000; // DWORD

pub const WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL: c_ulong = 133; // DWORD
pub const WINHTTP_PROTOCOL_FLAG_HTTP2: c_ulong = 1; // DWORD
pub const WINHTTP_OPTION_DECOMPRESSION: c_ulong = 118; // DWORD
pub const WINHTTP_DECOMPRESSION_FLAG_GZIP: c_ulong = 1; // DWORD
pub const WINHTTP_DECOMPRESSION_FLAG_DEFLATE: c_ulong = 2; // DWORD
