use std::{
    io::{ self, Error },
    os::{
        raw::*,
        windows::io::RawHandle,
    },
};

use crate::{ HttpHandle, Session };

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    fn WinHttpConnect(
        h_session: RawHandle,
        pswz_server_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
        n_server_port: c_ushort, // INTERNET_PORT -> WORD
        dw_reserved: c_ulong,
    ) -> RawHandle;
    
}

pub struct Connection {
    pub handle: HttpHandle,
}

impl Connection {
    
    pub fn new(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let handle = unsafe {
            
            let result = WinHttpConnect(
                session.handle.as_raw(),
                chikuwa::win_string(host).as_ptr(),
                port,
                0,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            HttpHandle::new(result)
            
        };
        
        Ok(Self {
            handle,
        })
    }
    
}
