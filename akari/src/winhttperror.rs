use std::{
    ffi::OsString,
    io::Error,
    os::{
        raw::*,
        windows::ffi::OsStringExt,
    },
    ptr,
};

unsafe extern "system" {
    
    // https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew
    fn GetModuleHandleW(
        lp_module_name: *const c_ushort, // LPCWSTR -> WCHAR -> wchar_t
    ) -> *mut c_void; // HMODULE
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-formatmessagew
    fn FormatMessageW(
        dw_flags: c_ulong, // DWORD
        lp_source: *const c_void, // LPCVOID
        dw_message_id: c_ulong, // DWORD
        dw_language_id: c_ulong, // DWORD
        lp_buffer: *mut c_ushort, // LPWSTR -> WCHAR -> wchar_t
        n_size: c_ulong, // DWORD,
        arguments: *mut c_char, // va_list
    ) -> c_ulong;
    
}

const MODULE_NAME: &str = "winhttp.dll";

const FORMAT_MESSAGE_FROM_SYSTEM: u32 = 0x0000_1000;
const FORMAT_MESSAGE_FROM_HMODULE: u32 = 0x0000_0800;
const FORMAT_MESSAGE_IGNORE_INSERTS: u32 = 0x0000_0200;

pub fn last_winhttp_error() -> Error {
    let os_error = Error::last_os_error();
    
    let error_code = os_error
        .raw_os_error()
        .unwrap_or(0)
        .cast_unsigned();
    
    // -------------------- get module --------------------
    
    let module = unsafe {
        
        GetModuleHandleW(
            chikuwa::win_string(MODULE_NAME).unwrap().as_ptr(),
        )
        
    };
    
    // -------------------- format message --------------------
    
    // FORMAT_MESSAGE_FROM_SYSTEM -> use core windows system message table as fallback
    // FORMAT_MESSAGE_IGNORE_INSERTS -> ignore insertion sequences, since "arguments" is null
    let mut flag = FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
    
    // FORMAT_MESSAGE_FROM_HMODULE -> use message table from provided module
    if ! module.is_null() {
        flag |= FORMAT_MESSAGE_FROM_HMODULE;
    }
    
    let mut buffer: [c_ushort; 1024] = [0; 1024];
    
    #[allow(clippy::cast_possible_truncation)]
    let buffer_length = buffer.len() as c_ulong;
    
    let length = unsafe {
        
        let result = FormatMessageW(
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
