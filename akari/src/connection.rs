use std::io;

use crate::{
    sys, last_winhttp_error,
    WinHttpHandle, Session,
};

pub struct Connection {
    pub handle: WinHttpHandle,
}

impl Connection {
    
    pub fn open(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let wide_host = chikuwa::win_string(host)?;
        
        let handle = unsafe {
            
            let result = sys::WinHttpConnect(
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
