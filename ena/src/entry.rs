use std::{
    ffi::OsStr,
    fs,
    io::{ self, Error, ErrorKind },
    path::{ Component, Path, PathBuf },
    ptr,
};

use crate::sys;

pub struct FilesEntry<'r> {
    inner: PathBuf,
    root: &'r Path,
}

impl<'r> FilesEntry<'r> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(inner: PathBuf, root: &'r Path) -> Self {
        Self { inner, root }
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn relative(&self) -> &Path {
        &self.inner
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn to_path(self) -> PathBuf {
        self.root.join(self.inner)
    }
    
    pub fn move_to_folder(self, folder: impl AsRef<Path>) -> io::Result<()> {
        // -------------------- target_folder and current_folder --------------------
        
        let folder = folder.as_ref();
        
        let target_folder = if folder.as_os_str().is_empty() {
            
            OsStr::new("")
            
        } else {
            
            let mut components = folder.components();
            
            // prevent path traversals and nested paths
            let (Some(Component::Normal(name)), None) = (components.next(), components.next()) else {
                return Err(Error::new(ErrorKind::InvalidInput, "Invalid target folder"));
            };
            
            // make sure folder name is a valid NTFS filename
            let Some(sanitized) = name.to_str().and_then(chikuwa::win_filename) else {
                return Err(Error::new(ErrorKind::InvalidInput, "Invalid target folder"));
            };
            
            if name != OsStr::new(&sanitized) {
                return Err(Error::new(ErrorKind::InvalidInput, "Invalid target folder"));
            }
            
            name
            
        };
        
        let current_folder = self.inner.parent()
            .unwrap_or(Path::new(""))
            .as_os_str();
        
        // if names only differ in case:
        // on (default) case-insensitive directories `MoveFileExW` treats it as a no-op (permission problems might still be encountered)
        // on case-sensitive directories the file is moved
        if target_folder == current_folder {
            return Ok(());
        }
        
        // -------------------- target_path and current_path --------------------
        
        let file_name = self.inner
            .file_name()
            .expect("Invalid entry file name");
        
        // if an empty folder name was provided, move entry to root folder
        // otherwise, attempt to create directory
        let target_path = if target_folder.is_empty() {
            
            self.root.join(file_name)
            
        } else {
            
            let intermediate_path = self.root.join(target_folder);
            fs::create_dir_all(&intermediate_path)?;
            
            intermediate_path.join(file_name)
            
        };
        
        let current_path = self.root.join(&self.inner);
        
        let target_path_wide = chikuwa::win_os_string(target_path.as_os_str())?;
        let current_path_wide = chikuwa::win_os_string(current_path.as_os_str())?;
        
        // -------------------- move --------------------
        
        // SAFETY:
        // 1. `current_path_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 2. `target_path_wide.as_ptr()` points to a null-terminated wide string that remains alive and immutable for the duration of the call
        // 3. `dw_flags` as `0` is valid and selects standard Win32 defaults
        let result = unsafe {
            
            sys::MoveFileExW(
                current_path_wide.as_ptr(),
                target_path_wide.as_ptr(),
                0, // no flags
            )
            
        };
        
        if result == 0 {
            return Err(Error::last_os_error());
        }
        
        // -------------------- response --------------------
        
        Ok(())
    }
    
    pub fn trash(self) -> io::Result<()> {
        let mut from_wide = chikuwa::win_os_string(self.to_path().as_os_str())?;
        // `SHFileOperationW` requires a double-null-terminated `p_from` list
        from_wide.push(0);
        
        // `FOF_SILENT`: hide progress dialog
        // `FOF_ALLOWUNDO`: preserve undo information in "explorer.exe", if possible
        // `FOF_NOERRORUI`: hide error dialog
        // `FOF_WANTNUKEWARNING`: display confirmation dialog if an item is going to be permanently deleted
        let flags = sys::FOF_SILENT | sys::FOF_ALLOWUNDO | sys::FOF_NOERRORUI | sys::FOF_WANTNUKEWARNING;
        
        let mut op = sys::ShFileOpStructW {
            hwnd: ptr::null_mut(), // no owner window for dialogs
            w_func: sys::FO_DELETE, // delete the files specified in `p_from`
            p_from: from_wide.as_ptr(),
            p_to: ptr::null(), // unused since `FO_DELETE` is set
            f_flags: flags,
            f_any_operations_aborted: 0,
            h_name_mappings: ptr::null_mut(), // unused since `FOF_WANTMAPPINGHANDLE` flag is unset
            lpsz_progress_title: ptr::null(), // unused since `FOF_SIMPLEPROGRESS` flag is unset
        };
        
        // SAFETY:
        // 1. `&raw mut op` points to a properly initialized `ShFileOpStructW`
        // 2. `op.p_from` points to a double-null-terminated wide string that remains alive and immutable for the duration of the call
        let result = unsafe {
            
            sys::SHFileOperationW(
                &raw mut op,
            )
            
        };
        
        if result != 0 {
            
            // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shfileoperationw
            let error = match result {
                0x74 => Error::new(ErrorKind::InvalidInput, "Cannot trash a root directory"),
                0x75 => Error::other("Trash operation cancelled"),
                0x78 => Error::new(ErrorKind::PermissionDenied, "Access denied to source path"),
                0x79 | 0x81 | 0xB7 => Error::new(ErrorKind::InvalidInput, "Path exceeds MAX_PATH"),
                0x7C => Error::new(ErrorKind::InvalidInput, "Invalid source path"),
                0x85 => Error::other("File is too large for the Recycle Bin"),
                0x86..=0x88 => Error::new(ErrorKind::PermissionDenied, "Source is on read-only or optical media"),
                0x10000 => Error::other("Unspecified error on Recycle Bin"),
                code => Error::from_raw_os_error(code),
            };
            
            return Err(error);
            
        }
        
        if op.f_any_operations_aborted != 0 {
            return Err(Error::other("Trash operation aborted"));
        }
        
        Ok(())
    }
    
}
