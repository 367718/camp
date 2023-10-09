use std::io;

use super::{ ffi, Session };

pub struct Connection {
    pub handle: ffi::HINTERNET,
}

impl Connection {
    
    pub fn new(session: &Session, host: &str, port: u16) -> io::Result<Self> {
        let handle = unsafe {
            
            let result = ffi::WinHttpConnect(
                session.handle,
                chikuwa::WinString::from(host).as_ptr(),
                port,
                0,
            );
            
            if result.is_null() {
                return Err(io::Error::last_os_error());
            }
            
            result
            
        };
        
        Ok(Self { handle })
    }
    
}

impl Drop for Connection {
    
    fn drop(&mut self) {
        unsafe {
            
            ffi::WinHttpCloseHandle(self.handle);
            
        }
    }
    
}
