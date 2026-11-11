use std::{
    io::{ self, Error, ErrorKind },
    os::raw::c_void,
    ptr,
};

use crate::{ sys, Request };

// INVARIANT: drop order required by http.sys, do not rearrange
// url_group -> queue -> session -> api
pub struct Server {
    #[allow(dead_code)]
    url_group: UrlGroup,
    queue: Queue,
    #[allow(dead_code)]
    session: Session,
    #[allow(dead_code)]
    api: Api,
}

struct Api;

struct Session {
    id: u64, // HTTP_OPAQUE_ID
}

struct Queue { 
    handle: *mut c_void, // HANDLE
}

struct UrlGroup {
    id: u64, // HTTP_OPAQUE_ID
}


// -------------------- server --------------------


impl Server {
    
    pub fn bind(url: &str) -> io::Result<Self> {
        let api = Api::init()?;
        let session = Session::new()?;
        let queue = Queue::new()?;
        let url_group = UrlGroup::new(&session, &queue, url)?;
        
        Ok(Self { url_group, queue, session, api })
    }
    
    pub fn accept(&self) -> io::Result<Request<'_>> {
        Request::new(self)
    }
    
    pub(crate) fn queue_handle(&self) -> *mut c_void {
        self.queue.handle
    }
    
}


// -------------------- api --------------------


impl Api {
    
    fn init() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        // perform initialization for applications that use the HTTP server API
        let flags = sys::HTTP_INITIALIZE_SERVER;
        
        // SAFETY:
        // 1. `version` is a valid `HttpApiVersion` struct
        // 2. `flags` is a valid option defined by the http.sys API
        // 3. `p_reserved` as `null` is required by the http.sys API
        let result = unsafe {
            
            sys::HttpInitialize(
                version,
                flags,
                ptr::null_mut(), // reserved
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(Self {})
    }
    
}

impl Drop for Api {
    
    fn drop(&mut self) {
        // release all resources used by server applications
        let flags = sys::HTTP_INITIALIZE_SERVER;
        
        // SAFETY:
        // 1. `flags` matches the flags previously specified in `HttpInitialize`
        // 2. `p_reserved` as `null` is required by the http.sys API
        // 3. `drop` runs only once, when the struct goes out of scope, preventing double-terminate bugs
        unsafe {
            
            sys::HttpTerminate(
                flags,
                ptr::null_mut(), // reserved
            )
            
        };
    }
    
}


// -------------------- session --------------------


impl Session {
    
    fn new() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        let mut id: u64 = 0;
        
        // SAFETY:
        // 1. `version` is a valid `HttpApiVersion` struct
        // 2. `&raw mut id` points to a live, properly aligned `u64` valid for writes
        // 3. `reserved` as `0` is required by the http.sys API
        let result = unsafe {
            
            sys::HttpCreateServerSession(
                version,
                &raw mut id,
                0, // reserved
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(Self { id })
    }
    
}

impl Drop for Session {
    
    fn drop(&mut self) {
        // SAFETY:
        // 1. `self.id` is a valid, active `HTTP_OPAQUE_ID` created by `HttpCreateServerSession` that has not been closed already
        // 2. `drop` runs only once, when the struct goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::HttpCloseServerSession(
                self.id,
            )
            
        };
    }
    
}


// -------------------- queue --------------------


impl Queue {
    
    fn new() -> io::Result<Self> {
        // use server API introduced in Windows Vista; previous version is deprecated
        let version = sys::HTTPAPI_VERSION_2;
        
        let mut handle = ptr::null_mut();
        
        // SAFETY:
        // 1. `version` is a valid `HttpApiVersion` struct
        // 2. `p_name` as `null` is valid and selects standard http.sys defaults
        // 3. `p_security_attributes` as `null` is valid and selects standard http.sys defaults
        // 4. `flags` as `0` is valid and selects standard http.sys defaults
        // 5. `&raw mut handle` is a live, properly aligned pointer, valid for writing a `HANDLE`
        let result = unsafe {
            
            sys::HttpCreateRequestQueue(
                version,
                ptr::null(), // queue cannot be accessed by name
                ptr::null(), // disable child process handle inheritance
                0, // create a new, standard worker queue
                &raw mut handle,
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(Self { handle })
    }
    
}

impl Drop for Queue {
    
    fn drop(&mut self) {
        // SAFETY:
        // 1. `self.handle` is guaranteed to be a valid, non-null queue handle initialized by the `HttpCreateRequestQueue` function
        // 2. ownership of the raw handle belongs exclusively to this `Queue` instance
        // 3. `drop` runs only once, when the struct goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::HttpCloseRequestQueue(
                self.handle,
            )
            
        };
    }
    
}


// -------------------- url_group --------------------


impl UrlGroup {
    
    fn new(session: &Session, queue: &Queue, url: &str) -> io::Result<Self> {
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
            flags: 1,
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
