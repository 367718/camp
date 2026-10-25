use std::{
    io::{ self, Error, ErrorKind },
    mem,
    os::raw::*,
};

use crate::sys;

pub struct UrlGroup {
    id: c_ulonglong,
}

impl UrlGroup {
    
    pub fn new(session_id: c_ulonglong, queue_handle: *mut c_void, url: &str) -> io::Result<Self> {
        if ! url.ends_with('/') {
            return Err(Error::new(ErrorKind::InvalidInput, "URL prefix must end with a trailing slash"));
        }
        
        // -------------------- create group --------------------
        
        let mut id: c_ulonglong = 0;
        
        unsafe {
            
            let result = sys::HttpCreateUrlGroup(
                session_id,
                &raw mut id,
                0,
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        }
        
        // early binding to make sure "HttpCloseUrlGroup" is called in case of error
        let url_group = Self { id };
        
        // -------------------- bind queue --------------------
        
        let binding = sys::HttpBindingInfo {
            flags: sys::HttpPropertyFlags { present: 1 },
            request_queue_handle: queue_handle,
        };
        
        unsafe {
            
            let result = sys::HttpSetUrlGroupProperty(
                url_group.id,
                sys::HTTP_SERVER_BINDING_PROPERTY,
                (&raw const binding).cast::<c_void>(),
                c_ulong::try_from(mem::size_of::<sys::HttpBindingInfo>()).expect("Unexpected truncation"),
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        }
        
        // -------------------- add url --------------------
        
        let wide_url = chikuwa::win_string(url)?;
        
        unsafe {
            
            let result = sys::HttpAddUrlToUrlGroup(
                url_group.id,
                wide_url.as_ptr(),
                0,
                0,
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        }
        
        Ok(url_group)
    }
    
}

impl Drop for UrlGroup {
    
    fn drop(&mut self) {
        unsafe {
            
            sys::HttpCloseUrlGroup(
                self.id,
            );
            
        }
    }
    
}
