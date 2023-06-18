use std::{
    error::Error,
    io,
    os::raw::*,
    ptr,
};

use crate::WinString;

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew
        pub fn ShellExecuteW(
            hwnd: *mut c_void, // HWND
            lpoperation: *const c_ushort,
            lpfile: *const c_ushort,
            lpparameters: *const c_ushort,
            lpdirectory: *const c_ushort,
            nshowcmd: c_int,
        ) -> *mut c_void; // HINSTANCE
        
    }
    
    pub const SW_NORMAL: c_int = 1;
    
}

pub fn execute_app(path: &str) -> Result<(), Box<dyn Error>> {
    unsafe {
        
        let result = ffi::ShellExecuteW(
            ptr::null_mut(),
            WinString::from("open").as_ptr(),
            WinString::from(path).as_ptr(),
            ptr::null(),
            ptr::null(),
            ffi::SW_NORMAL,
        );
        
        if result as isize <= 32 {
            return Err(io::Error::last_os_error().into());
        }
        
    }
        
    Ok(())
}
