use std::{
    io::{ self, Error, ErrorKind },
    os::raw::c_void,
};

use crate::sys;
use super::{ Session, Queue };

pub(super) struct UrlGroup {
    id: u64, // HTTP_OPAQUE_ID
}

impl UrlGroup {
    
    pub(super) fn new(session: &Session, queue: &Queue, url: &str) -> io::Result<Self> {
        if ! url.ends_with('/') {
            return Err(Error::new(ErrorKind::InvalidInput, "URL prefix must end with a trailing slash"));
        }
        
        let url_wide = chikuwa::win_string(url)?;
        
        // -------------------- create group --------------------
        
        let mut id: u64 = 0;
        
        // SAFETY:
        // 1. `session.id` is a valid `HTTP_OPAQUE_ID` whose lifetime is guaranteed by the borrowed `&Session`
        // 2. `&raw mut id` points to a live, properly aligned `u64` valid for writes
        // 3. `reserved` as `0` is required by the http.sys API
        let result = unsafe {
            
            sys::HttpCreateUrlGroup(
                session.id,
                &raw mut id,
                0, // reserved
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        // early binding to make sure `HttpCloseUrlGroup` is called in case of error
        let url_group = Self { id };
        
        // -------------------- bind queue --------------------
        
        let binding = sys::HttpBindingInfo {
            flags: sys::HttpPropertyFlags { present: 1 },
            request_queue_handle: queue.handle,
        };
        
        // modifies or sets the url group association with a request queue
        let property = sys::HTTP_SERVER_BINDING_PROPERTY;
        
        // SAFETY:
        // 1. `url_group.id` is a valid, active `HTTP_OPAQUE_ID` created by `HttpCreateUrlGroup` that has not been closed already
        // 2. `property` is a valid `HTTP_SERVER_PROPERTY` variant
        // 3. `(&raw const binding).cast::<c_void>()` points to a live, properly aligned `HttpBindingInfo` struct
        // 4. `binding.request_queue_handle` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&Queue`
        // 5. `size_of_val(&binding)` matches the size in bytes of `binding`, preventing out-of-bounds reads
        let result = unsafe {
            
            sys::HttpSetUrlGroupProperty(
                url_group.id,
                property,
                (&raw const binding).cast::<c_void>(),
                u32::try_from(size_of_val(&binding)).expect("Unexpected truncation"),
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        // -------------------- add url --------------------
        
        // SAFETY:
        // 1. `url_group.id` is a valid, active `HTTP_OPAQUE_ID` created by `HttpCreateUrlGroup` that has not been closed already
        // 2. `url_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. `url_context` as `0` is valid and selects standard http.sys defaults
        // 4. `reserved` as `0` is required by the http.sys API
        let result = unsafe {
            
            sys::HttpAddUrlToUrlGroup(
                url_group.id,
                url_wide.as_ptr(),
                0, // no context
                0, // reserved
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(url_group)
    }
    
}

impl Drop for UrlGroup {
    
    fn drop(&mut self) {
        // SAFETY:
        // 1. `self.id` is a valid, active `HTTP_OPAQUE_ID` created by `HttpCreateUrlGroup` that has not been closed already
        // 2. `drop` runs only once, when the struct goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::HttpCloseUrlGroup(
                self.id,
            )
            
        };
    }
    
}
