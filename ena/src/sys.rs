use std::{
    ffi::c_void,
    os::windows::raw::HANDLE,
};

#[link(name = "kernel32")]
unsafe extern "system" {
    
    // -------------------- readdir --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirstfileexw
    pub fn FindFirstFileExW(
        lp_file_name: *const u16, // LPCWSTR
        f_info_level_id: u32, // FINDEX_INFO_LEVELS
        lp_find_file_data: *mut Win32FindDataW,
        f_search_op: u32, // FINDEX_SEARCH_OPS
        lp_search_filter: *const c_void,
        dw_additional_flags: u32, // DWORD
    ) -> HANDLE;
    
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findnextfilew
    pub fn FindNextFileW(
        h_find_file: HANDLE,
        lp_find_file_data: *mut Win32FindDataW,
    ) -> i32; // BOOL
    
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findclose
    pub fn FindClose(
        h_find_file: HANDLE,
    ) -> i32; // BOOL
    
    // -------------------- entry --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw
    pub fn MoveFileExW(
        lp_existing_file_name: *const u16, // LPCWSTR
        lp_new_file_name: *const u16, // LPCWSTR
        dw_flags: u32, // DWORD
    ) -> i32; // BOOL
    
    // -------------------- mark --------------------
    
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileattributesw
    pub fn GetFileAttributesW(
        lp_file_name: *const u16,
    ) -> u32;
    
}

#[repr(C)]
pub struct Win32FindDataW {
    pub attrs: u32, // DWORD
    pub creation: [u32; 2], // FILETIME
    pub last_access: [u32; 2], // FILETIME
    pub last_write: [u32; 2], // FILETIME
    pub size_high: u32, // DWORD
    pub size_low: u32, // DWORD
    pub reserved0: u32, // DWORD
    pub reserved1: u32, // DWORD
    pub file_name: [u16; 260], // WCHAR
    pub alt_file_name: [u16; 14], // WCHAR
}

pub const INVALID_HANDLE_VALUE: HANDLE = -1_isize as HANDLE;
pub const INVALID_FILE_ATTRIBUTES: u32 = 0xFFFF_FFFF; // DWORD

pub const FIND_EX_INFO_BASIC: u32 = 1;
pub const FIND_EX_SEARCH_NAME_MATCH: u32 = 0;
pub const FIND_FIRST_EX_LARGE_FETCH: u32 = 2; // DWORD

pub const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x0010; // DWORD
pub const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400; // DWORD

pub const IO_REPARSE_TAG_NAME_SURROGATE: u32 = 0x2000_0000;
