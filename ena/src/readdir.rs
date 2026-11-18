use std::{
    ffi::OsString,
    mem,
    os::windows::{
        ffi::{ OsStrExt, OsStringExt },
        raw::HANDLE,
    },
    path::Path,
    ptr,
};

use crate::sys;

pub struct ReadDir {
    handle: HANDLE,
    data: sys::Win32FindDataW,
    first: bool,
}

pub enum EntryKind {
    File,
    Dir,
}

const DOT: u16 = b'.' as u16;

impl ReadDir {
    
    pub fn open(dir_path: &Path) -> Option<Self> {
        // -------------------- pattern construction --------------------
        
        let dir_path_as_os_str = dir_path.as_os_str();
        
        // pattern must be `dir_path` + `\*` + `NUL`
        let mut pattern = Vec::with_capacity(dir_path_as_os_str.len() + 3);
        
        pattern.extend(dir_path_as_os_str.encode_wide());
        
        if pattern.last() != Some(&u16::from(b'\\')) {
            pattern.push(u16::from(b'\\'));
        }
        
        pattern.push(u16::from(b'*'));
        pattern.push(0);
        
        // -------------------- handle creation --------------------
        
        // speed up directory enumeration by skipping legacy 8.3 short file names querying
        let info_level = sys::FIND_EX_INFO_BASIC;
        
        // SAFETY: all-zero bit patterns are valid for structs that consist entirely of integer primitives
        let mut data = unsafe { mem::zeroed::<sys::Win32FindDataW>() };
        
        // search for entries matching a specified file name or pattern
        let search_op = sys::FIND_EX_SEARCH_NAME_MATCH;
        
        // use larger internal buffers to reduce round trips and improve directory enumeration performance
        let additional_flags = sys::FIND_FIRST_EX_LARGE_FETCH;
        
        // SAFETY:
        // 1. `pattern.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 2. `info_level` is a valid option defined by the Win32 API
        // 3. `&raw mut data` points to a zero-initialized, properly aligned `Win32FindDataW` valid for writing
        // 4. `search_op` is a valid option defined by the Win32 API
        // 5. `lp_search_filter` as `null` is required by the Win32 API
        // 6. `additional_flags` is a valid option defined by the Win32 API
        let handle = unsafe {
            
            sys::FindFirstFileExW(
                pattern.as_ptr(),
                info_level,
                &raw mut data,
                search_op,
                ptr::null(),
                additional_flags,
            )
            
        };
        
        if handle == sys::INVALID_HANDLE_VALUE {
            return None;
        }
        
        // -------------------- response --------------------
        
        Some(Self { handle, data, first: true })
    }
    
}

impl Iterator for ReadDir {
    
    type Item = (OsString, EntryKind);
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            
            // -------------------- next file retrieval --------------------
            
            if self.first {
                
                self.first = false;
                
            } else {
                
                // SAFETY:
                // 1. `self.handle` is guaranteed to be a valid, non-null find-search handle initialized by the `FindFirstFileExW` function
                // 2. `&raw mut self.data` points to a properly aligned `Win32FindDataW` valid for writing
                let result = unsafe {
                    
                    sys::FindNextFileW(
                        self.handle,
                        &raw mut self.data,
                    )
                    
                };
                
                // no more entries to be found in this directory
                if result == 0 {
                    return None;
                }
                
            }
            
            // -------------------- current directory and parent directory check --------------------
            
            if matches!(&self.data.file_name, [DOT, 0, ..] | [DOT, DOT, 0, ..]) {
                continue;
            }
            
            // -------------------- symlink and junction check --------------------
            
            // a filesystem driver attached metadata to this entry
            if (self.data.attrs & sys::FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
                
                // when `FILE_ATTRIBUTE_REPARSE_POINT` is set, `reserved0` holds the reparse tag
                // the reparse tag holds the surrogate bit, which marks path-redirecting links (symlinks, junctions, etc.)
                if (self.data.reserved0 & sys::IO_REPARSE_TAG_NAME_SURROGATE) != 0 {
                    continue;
                }
                
            }
            
            // -------------------- name retrieval --------------------
            
            // on NTFS, an individual file or folder name can never exceed 255 UTF-16 code units
            // `self.data.file_name` is more than big enough, as it is defined as `[u16; 260]`
            let file_name_length = self.data.file_name.iter()
                .position(|&code_unit| code_unit == 0)
                .unwrap_or(self.data.file_name.len());
            
            let file_name = &self.data.file_name[..file_name_length];
            
            // -------------------- kind identification --------------------
            
            let is_directory = (self.data.attrs & sys::FILE_ATTRIBUTE_DIRECTORY) != 0;
            
            let kind = if is_directory {
                EntryKind::Dir
            } else {
                EntryKind::File
            };
            
            // -------------------- response --------------------
            
            return Some((OsString::from_wide(file_name), kind));
            
        }
    }
    
}

impl Drop for ReadDir {
    
    fn drop(&mut self) {
        // SAFETY:
        // 1. `self.handle` is guaranteed to be a valid, non-null find-search handle initialized by the `FindFirstFileExW` function
        // 2. ownership of the raw handle belongs exclusively to this `ReadDir` instance
        // 3. `drop` runs only once, when the struct goes out of scope, preventing double-close bugs
        unsafe {
            
            sys::FindClose(
                self.handle,
            )
            
        };
    }
    
}
