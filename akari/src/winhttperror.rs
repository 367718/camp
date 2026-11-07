use std::{
    io::Error,
    os::raw::*,
    ptr,
};

use crate::sys;

// "winhttp.dll\0"
const MODULE_NAME: &[u16] = &[119, 105, 110, 104, 116, 116, 112, 46, 100, 108, 108, 0];

pub fn last_winhttp_error() -> Error {
    // capture last error early to prevent later calls from overwriting it
    let os_error = Error::last_os_error();
    
    // -------------------- get message id --------------------
    
    let message_id = os_error
        .raw_os_error()
        .unwrap_or(0)
        .cast_unsigned();
    
    // -------------------- get source --------------------
    
    // SAFETY:
    // 1. `MODULE_NAME.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
    // 2. on failure, `null` is returned and subsequently checked before using the `FORMAT_MESSAGE_FROM_HMODULE` flag
    let source = unsafe {
        
        sys::GetModuleHandleW(
            MODULE_NAME.as_ptr(),
        )
        
    };
    
    // -------------------- format message --------------------
    
    // `FORMAT_MESSAGE_FROM_SYSTEM`: use core windows system message table as fallback
    // `FORMAT_MESSAGE_IGNORE_INSERTS`: ignore insertion sequences, since `arguments` is null
    let mut flag = sys::FORMAT_MESSAGE_FROM_SYSTEM | sys::FORMAT_MESSAGE_IGNORE_INSERTS;
    
    if ! source.is_null() {
        // use message table from provided module
        flag |= sys::FORMAT_MESSAGE_FROM_HMODULE;
    }
    
    let mut buffer: [c_ushort; 1024] = [0; 1024];
    
    // SAFETY:
    // 1. `source.cast_const()` is either `null` or a valid module handle to `winhttp.dll` which remains loaded for the duration of the process
    // 2. `message_id` represents the most recent operating system error that occurred on the current thread
    // 3. `dw_language_id` as `0` is valid and selects the default language search sequence
    // 4. `buffer.as_mut_ptr()` points to a live, properly aligned stack array valid for writes
    // 5. `n_size` as `buffer.len()` specifies the capacity in characters rather than bytes, preventing out-of-bounds writes
    // 6. `arguments` as `null` is valid as long as `FORMAT_MESSAGE_IGNORE_INSERTS` is set
    // 7. on success, the returned length cannot exceed `n_size`, ensuring slice indexing into `buffer` is within bounds
    let result = unsafe {
        
        sys::FormatMessageW(
            flag,
            source.cast_const(),
            message_id,
            0, // default language search sequence
            buffer.as_mut_ptr(),
            c_ulong::try_from(buffer.len()).expect("Unexpected truncation"),
            ptr::null_mut(), // no arguments
        )
        
    };
    
    // formatting failed; return the default standard library error
    if result == 0 {
        return os_error;
    }
    
    let length = result;
    let message = String::from_utf16_lossy(&buffer[..length as usize]);
    
    Error::other(format!("{} (OS Error {})", message.trim_end(), message_id))
}
