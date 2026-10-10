use std::{
    io::{ self, Error, ErrorKind, Read },
    os::{
        raw::*,
        windows::io::{ RawHandle, AsRawHandle },
    },
    ptr,
};

use super::{ WinHttpHandle, Connection };

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
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreceiveresponse
    fn WinHttpReceiveResponse(
        h_request: RawHandle,
        lp_reserved: *mut c_void, // LPVOID
    ) -> c_int; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpreaddata
    fn WinHttpReadData(
        h_request: RawHandle,
        lp_buffer: *mut c_void, // LPVOID
        dw_number_of_bytes_to_read: c_ulong, // DWORD
        lpdw_number_of_bytes_read: *mut c_ulong, // LPDWORD -> DWORD
    ) -> c_int; // BOOL
    
}

const WINHTTP_FLAG_SECURE: c_ulong = 0x0080_0000; // DWORD
const WINHTTP_NO_ADDITIONAL_HEADERS: *const c_ushort = ptr::null(); // LPCWSTR -> WCHAR -> wchar_t

pub struct Response {
    handle: WinHttpHandle,
}

impl Response {
    
    pub(crate) fn receive(connection: &Connection, path: &str) -> io::Result<Self> {
        // -------------------- open request --------------------
        
        let handle = unsafe {
            
            let result = WinHttpOpenRequest(
                connection.handle.as_raw_handle(),
                ptr::null(),
                chikuwa::win_string(path)?.as_ptr(),
                ptr::null(),
                ptr::null(),
                ptr::null_mut(),
                WINHTTP_FLAG_SECURE,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            WinHttpHandle::from(result)
            
        };
        
        // -------------------- send request --------------------
        
        unsafe {
            
            let result = WinHttpSendRequest(
                handle.as_raw_handle(),
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
        
        // -------------------- receive response --------------------
        
        unsafe {
            
            let result = WinHttpReceiveResponse(
                handle.as_raw_handle(),
                ptr::null_mut(),
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

impl Read for Response {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut amount_read: c_ulong = 0;
        
        let bytes = c_ulong::try_from(buf.len())
            .map_err(|_| Error::from(ErrorKind::InvalidInput))?;
        
        unsafe {
            
            let result = WinHttpReadData(
                self.handle.as_raw_handle(),
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
