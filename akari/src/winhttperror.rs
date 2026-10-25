use std::{
    ffi::OsString,
    io::Error,
    os::{
        raw::*,
        windows::ffi::OsStringExt,
    },
    ptr,
};

use crate::sys;

const MODULE_NAME: &str = "winhttp.dll";

pub fn last_winhttp_error() -> Error {
    let os_error = Error::last_os_error();
    
    let error_code = os_error
        .raw_os_error()
        .unwrap_or(0)
        .cast_unsigned();
    
    // -------------------- get module --------------------
    
    let wide_name = chikuwa::win_string(MODULE_NAME).unwrap();
    
    let module = unsafe {
        
        sys::GetModuleHandleW(
            wide_name.as_ptr(),
        )
        
    };
    
    // -------------------- format message --------------------
    
    // FORMAT_MESSAGE_FROM_SYSTEM -> use core windows system message table as fallback
    // FORMAT_MESSAGE_IGNORE_INSERTS -> ignore insertion sequences, since "arguments" is null
    let mut flag = sys::FORMAT_MESSAGE_FROM_SYSTEM | sys::FORMAT_MESSAGE_IGNORE_INSERTS;
    
    // FORMAT_MESSAGE_FROM_HMODULE -> use message table from provided module
    if ! module.is_null() {
        flag |= sys::FORMAT_MESSAGE_FROM_HMODULE;
    }
    
    let mut buffer: [c_ushort; 1024] = [0; 1024];
    
    #[allow(clippy::cast_possible_truncation)]
    let buffer_length = buffer.len() as c_ulong;
    
    let length = unsafe {
        
        let result = sys::FormatMessageW(
            flag,
            module.cast_const(),
            error_code,
            0,
            buffer.as_mut_ptr(),
            buffer_length,
            ptr::null_mut(),
        );
        
        // formatting failed; return the default standard library error
        if result == 0 {
            return os_error;
        }
        
        result
        
    };
    
    let message = OsString::from_wide(&buffer[..length as usize]);
    let message_string = message.to_string_lossy();
    
    Error::other(format!("{} (OS Error {})", message_string.trim_end(), error_code))
}
