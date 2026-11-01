use std::{
    io::{ self, Error },
    os::raw::*,
};

use crate::sys;

pub struct Session {
    pub(crate) id: c_ulonglong, // HTTP_OPAQUE_ID
}

impl Session {
    
    pub fn new() -> io::Result<Self> {
        // use server api introduced in Windows Vista; the previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        let mut server_session_id: c_ulonglong = 0;
        
        unsafe {
            
            let result = sys::HttpCreateServerSession(
                version,
                &raw mut server_session_id,
                0, // reserved
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        };
        
        Ok(Self { id: server_session_id })
    }
    
}

impl Drop for Session {
    
    fn drop(&mut self) {
        unsafe {
            
            sys::HttpCloseServerSession(
                self.id,
            );
            
        }
    }
    
}
