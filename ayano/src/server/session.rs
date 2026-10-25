use std::{
    io::{ self, Error },
    os::raw::*,
};

use crate::sys;

pub struct Session {
    pub(crate) id: c_ulonglong,
}

impl Session {
    
    pub fn new() -> io::Result<Self> {
        let mut id: c_ulonglong = 0;
        
        unsafe {
            
            let result = sys::HttpCreateServerSession(
                sys::HTTPAPI_VERSION_2,
                &raw mut id,
                0,
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        };
        
        Ok(Self { id })
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
