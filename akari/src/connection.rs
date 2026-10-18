use std::{
    io,
    os::raw::*,
};

use crate::{
    last_winhttp_error,
    WinHttpHandle,
    Session,
};

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winhttp/nf-winhttp-winhttpconnect
    fn WinHttpConnect(
        h_session: *mut c_void, // HINTERNET
        pswz_server_name: *const c_ushort, // LPCWSTR
        n_server_port: c_ushort, // INTERNET_PORT
        dw_reserved: c_ulong,
    ) -> *mut c_void; // HINTERNET
    
}

pub struct Connection {
    pub handle: WinHttpHandle,
}

impl Connection {
    
    pub fn open(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let wide_host = chikuwa::win_string(host)?;
        
        let handle = unsafe {
            
            let result = WinHttpConnect(
                session.handle.as_raw(),
                wide_host.as_ptr(),
                port,
                0,
            );
            
            if result.is_null() {
                return Err(last_winhttp_error());
            }
            
            WinHttpHandle::new(result)
            
        };
        
        Ok(Self {
            handle,
        })
    }
    
}
