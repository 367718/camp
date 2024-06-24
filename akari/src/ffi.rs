use std::{
    io::{ self, Error, ErrorKind },
    mem,
    os::raw::*,
    ptr,
};

use crate::{
    Handle, HandleSource,
    DNS_RESOLUTION_TIMEOUT_AS_MILLIS, CONNECTION_TIMEOUT_AS_MILLIS,
    SEND_TIMEOUT_AS_MILLIS, RECEIVE_TIMEOUT_AS_MILLIS,
};

extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpclosehandle
    fn WinHttpCloseHandle(
        h_internet: HINTERNET,
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopen
    fn WinHttpOpen(
        psz_agent_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_access_type: c_ulong, // DWORD
        psz_proxy_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        psz_proxy_bypass_w: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> HINTERNET;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsettimeouts
    fn WinHttpSetTimeouts(
        h_internet: HINTERNET,
        n_resolve_timeout: c_int,
        n_connect_timeout: c_int,
        n_send_timeout: c_int,
        n_receive_timeout: c_int,
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsetoption
    fn WinHttpSetOption(
        h_internet: HINTERNET,
        dw_option: c_ulong, // DWORD
        lp_buffer: *mut c_void, // LPVOID
        dw_buffer_length: c_ulong, // DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    fn WinHttpConnect(
        h_session: HINTERNET,
        pswz_server_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        n_server_port: c_ushort, // INTERNET_PORT -> WORD
        dw_reserved: c_ulong,
    ) -> HINTERNET;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpopenrequest
    fn WinHttpOpenRequest(
        h_connect: HINTERNET,
        pwsz_verb: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_object_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_version: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        pwsz_referrer: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        ppwsz_accept_types: *mut *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_flags: c_ulong, // DWORD
    ) -> HINTERNET;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpsendrequest
    fn WinHttpSendRequest(
        h_request: HINTERNET,
        lpsz_headers: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        dw_headers_length: c_ulong, // DWORD
        lp_optional: *mut c_void, // LPVOID
        dw_optional_length: c_ulong, // DWORD
        dw_total_length: c_ulong, // DWORD
        dw_context: usize, // DWORD_PTR -> ULONG_PTR
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    fn WinHttpReceiveResponse(
        h_request: HINTERNET,
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpqueryheaders
    fn WinHttpQueryHeaders(
        h_request: HINTERNET,
        dw_info_level: c_ulong, // DWORD
        pwsz_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        lp_buffer: *mut c_void, // LPVOID
        lpdw_buffer_length: *mut c_ulong, // LPDWORD -> DWORD
        lpdw_index: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    fn WinHttpReadData(
        h_request: HINTERNET,
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
}


// -------------------- handle --------------------


#[allow(clippy::upper_case_acronyms)]
pub type HINTERNET = *mut c_void;

pub fn close_handle(handle: &Handle) -> io::Result<()> {
    unsafe {
        
        let result = WinHttpCloseHandle(
            handle.as_raw(),
        );
        
        if result == 0 {
            return Err(Error::last_os_error());
        }
        
    }
    
    Ok(())
}


// -------------------- session --------------------


const WINHTTP_ACCESS_TYPE_DEFAULT_PROXY: c_ulong = 0; // DWORD
const WINHTTP_NO_PROXY_NAME: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t
const WINHTTP_NO_PROXY_BYPASS: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t

const WINHTTP_OPTION_ENABLE_HTTP_PROTOCOL: c_ulong = 133; // DWORD
const WINHTTP_PROTOCOL_FLAG_HTTP2: c_ulong = 1; // DWORD

pub fn open(user_agent: &str) -> io::Result<Handle> {
    let hinternet = unsafe {
        
        let result = WinHttpOpen(
            chikuwa::WinString::from(user_agent).as_ptr(),
            WINHTTP_ACCESS_TYPE_DEFAULT_PROXY,
            WINHTTP_NO_PROXY_NAME,
            WINHTTP_NO_PROXY_BYPASS,
            0,
        );
        
        if result.is_null() {
            return Err(Error::last_os_error());
        }
        
        result
        
    };
    
    Ok(Handle::new(hinternet, HandleSource::Session))
}

pub fn set_timeouts(handle: &Handle) -> io::Result<()> {
    if handle.source() != HandleSource::Session {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Session'"));
    }
    
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
    
    Ok(())
}

pub fn set_option(handle: &Handle) -> io::Result<()> {
    if handle.source() != HandleSource::Session {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Session'"));
    }
    
    let mut version = WINHTTP_PROTOCOL_FLAG_HTTP2;
    
    #[allow(clippy::cast_possible_truncation)]
    let bytes = mem::size_of::<c_ulong>() as c_ulong;
    
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
    
    Ok(())
}


// -------------------- connection --------------------



pub fn connect(handle: &Handle, host: &str, port: u16) -> io::Result<Handle> {
    if handle.source() != HandleSource::Session {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Session'"));
    }
    
    let hinternet = unsafe {
        
        let result = WinHttpConnect(
            handle.as_raw(),
            chikuwa::WinString::from(host).as_ptr(),
            port,
            0,
        );
        
        if result.is_null() {
            return Err(Error::last_os_error());
        }
        
        result
        
    };
    
    Ok(Handle::new(hinternet, HandleSource::Connection))
}


// -------------------- request --------------------


const WINHTTP_NO_REFERER: *const c_ushort = ptr::null(); // // LPCWSTR -> WCHAR -> wchar_t
const WINHTTP_DEFAULT_ACCEPT_TYPES: *mut *const c_ushort = ptr::null_mut(); // // LPCWSTR -> WCHAR -> wchar_t
const WINHTTP_FLAG_SECURE: c_ulong = 0x0080_0000; // DWORD

const WINHTTP_NO_ADDITIONAL_HEADERS: *const c_ushort = ptr::null(); // // LPCWSTR -> WCHAR -> wchar_t

pub fn open_request(handle: &Handle, path: &str, secure: bool) -> io::Result<Handle> {
    if handle.source() != HandleSource::Connection {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Connection'"));
    }
    
    let flags = if secure {
        WINHTTP_FLAG_SECURE
    } else {
        0
    };
    
    let hinternet = unsafe {
        
        let result = WinHttpOpenRequest(
            handle.as_raw(),
            ptr::null(),
            chikuwa::WinString::from(path).as_ptr(),
            ptr::null(),
            WINHTTP_NO_REFERER,
            WINHTTP_DEFAULT_ACCEPT_TYPES,
            flags,
        );
        
        if result.is_null() {
            return Err(Error::last_os_error());
        }
        
        result
        
    };
    
    Ok(Handle::new(hinternet, HandleSource::Request))
}

pub fn send_request(handle: &Handle) -> io::Result<()> {
    if handle.source() != HandleSource::Request {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Request'"));
    }
    
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
    
    Ok(())
}


// -------------------- response --------------------


const WINHTTP_QUERY_CONTENT_LENGTH: c_ulong = 5; // DWORD
const WINHTTP_QUERY_FLAG_NUMBER: c_ulong = 0x2000_0000; // DWORD
const WINHTTP_HEADER_NAME_BY_INDEX: *const c_ushort = ptr::null(); // // LPCWSTR -> WCHAR -> wchar_t
const WINHTTP_NO_HEADER_INDEX: *mut c_ulong = ptr::null_mut(); // // LPCWSTR -> WCHAR -> wchar_t

pub fn receive_response(handle: &Handle) -> io::Result<()> {
    if handle.source() != HandleSource::Request {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Request'"));
    }
    
    unsafe {
        
        let result = WinHttpReceiveResponse(
            handle.as_raw(),
            ptr::null_mut(),
        );
        
        if result == 0 {
            return Err(Error::last_os_error());
        }
        
    }
    
    Ok(())
}

pub fn query_headers(handle: &Handle) -> io::Result<usize> {
    if handle.source() != HandleSource::Request {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Request'"));
    }
    
    let mut content_length: c_ulong = 0;
    
    #[allow(clippy::cast_possible_truncation)]
    let mut bytes = mem::size_of::<c_ulong>() as c_ulong;
    
    unsafe {
        
        let result = WinHttpQueryHeaders(
            handle.as_raw(),
            WINHTTP_QUERY_CONTENT_LENGTH | WINHTTP_QUERY_FLAG_NUMBER,
            WINHTTP_HEADER_NAME_BY_INDEX,
            ptr::from_mut(&mut content_length).cast::<c_void>(),
            &mut bytes,
            WINHTTP_NO_HEADER_INDEX,
        );
        
        if result == 0 {
            return Err(Error::last_os_error());
        }
        
    }
    
    Ok(content_length as usize)
}

pub fn read_data(handle: &Handle, buf: &mut [u8]) -> io::Result<usize> {
    if handle.source() != HandleSource::Request {
        return Err(Error::new(ErrorKind::InvalidInput, "Handle source must be 'Request'"));
    }
    
    let mut amount_read: c_ulong = 0;
    
    #[allow(clippy::cast_possible_truncation)]
    let bytes = buf.len() as c_ulong;
    
    unsafe {
        
        let result = WinHttpReadData(
            handle.as_raw(),
            ptr::from_mut(buf).cast::<c_void>(),
            bytes,
            &mut amount_read,
        );
        
        if result == 0 {
            return Err(Error::last_os_error());
        }
        
    }
    
    Ok(amount_read as usize)
}
