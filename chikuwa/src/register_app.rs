use std::{
    error::Error,
    io,
    os::{
        raw::*,
        windows::raw::HANDLE,
    },
    ptr,
};

use crate::WinString;

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw
        pub fn CreateMutexW(
            lpmutexattributes: *const c_void, // SECURITY_ATTRIBUTES
            binitialowner: c_int,
            lpname: *const c_ushort,
        ) -> HANDLE;
        
    }
    
}

pub fn register_app(name: &str) -> Result<(), Box<dyn Error>> {
    // mutex will be automatically released on application shutdown
    unsafe {
        
        let _ = ffi::CreateMutexW(
            ptr::null(),
            false as c_int,
            WinString::from(name).as_ptr(),
        );
        
    }
    
    // allow app to run even if mutex could not be created
    if io::Error::last_os_error().kind() == io::ErrorKind::AlreadyExists {
        return Err("Only one instance of the application can be running at one time".into());
    }
    
    Ok(())
}
