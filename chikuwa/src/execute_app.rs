use std::{
    error::Error,
    io,
    os::raw::*,
    ptr,
};

extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecutew
    fn ShellExecuteW(
        hwnd: *mut c_void, // HWND
        lpoperation: *const c_ushort,
        lpfile: *const c_ushort,
        lpparameters: *const c_ushort,
        lpdirectory: *const c_ushort,
        nshowcmd: c_int,
    ) -> *mut c_void; // HINSTANCE
    
}

pub fn execute_app(path: &str) -> Result<(), Box<dyn Error>> {
    let encoded_operation: Vec<c_ushort> = "open".encode_utf16()
        .chain(Some(0))
        .collect();
    
    let encoded_path: Vec<c_ushort> = path.encode_utf16()
        .chain(Some(0))
        .collect();
    
    let result = unsafe {
        
        ShellExecuteW(
            ptr::null_mut(),
            encoded_operation.as_ptr(),
            encoded_path.as_ptr(),
            ptr::null(),
            ptr::null(),
            5, // SW_SHOW
        )
        
    };
    
    if result as isize <= 32 {
        return Err(io::Error::last_os_error().into());
    }
    
    Ok(())
}
