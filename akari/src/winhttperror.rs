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
    
    // -------------------- get message id --------------------
    
    let message_id = os_error
        .raw_os_error()
        .unwrap_or(0)
        .cast_unsigned();
    
    // -------------------- get source --------------------
    
    let wide_name = chikuwa::win_string(MODULE_NAME).unwrap();
    
    let source = unsafe {
        
        sys::GetModuleHandleW(
            wide_name.as_ptr(),
        )
        
    };
    
    // -------------------- format message --------------------
    
    // FORMAT_MESSAGE_FROM_SYSTEM: use core windows system message table as fallback
    // FORMAT_MESSAGE_IGNORE_INSERTS: ignore insertion sequences, since "arguments" is null
    let mut flag = sys::FORMAT_MESSAGE_FROM_SYSTEM | sys::FORMAT_MESSAGE_IGNORE_INSERTS;
    
    if ! source.is_null() {
        // use message table from provided module
        flag |= sys::FORMAT_MESSAGE_FROM_HMODULE;
    }
    
    let mut buffer: [c_ushort; 1024] = [0; 1024];
    
    let length = unsafe {
        
        let result = sys::FormatMessageW(
            flag,
            source.cast_const(),
            message_id,
            0, // default language
            buffer.as_mut_ptr(),
            c_ulong::try_from(buffer.len()).expect("Unexpected truncation"),
            ptr::null_mut(), // no arguments
        );
        
        // formatting failed; return the default standard library error
        if result == 0 {
            return os_error;
        }
        
        result
        
    };
    
    let message = OsString::from_wide(&buffer[..length as usize]);
    let message_string = message.to_string_lossy();
    
    Error::other(format!("{} (OS Error {})", message_string.trim_end(), message_id))
}
