use std::io::{ self, Error };

use super::{ ffi, Handle, Session };

pub struct Connection {
    pub handle: Handle,
}

impl Connection {
    
    pub fn new(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let handle = unsafe {
            
            let result = ffi::WinHttpConnect(
                session.handle.as_raw(),
                chikuwa::WinString::from(host).as_ptr(),
                port,
                0,
            );
            
            if result.is_null() {
                return Err(Error::last_os_error());
            }
            
            Handle::new(result)
            
        };
        
        Ok(Self {
            handle,
        })
    }
    
}
