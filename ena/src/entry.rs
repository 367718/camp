use std::{
    ffi::OsStr,
    fs,
    io::{ self, Error, ErrorKind },
    path::{ Component, Path, PathBuf },
};

use crate::{ sys, mark };

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
    
    pub fn is_marked(&self, flag: impl AsRef<OsStr>) -> io::Result<bool> {
        mark::is_marked(self.root, &self.inner, flag)
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn to_path(self) -> PathBuf {
        self.root.join(self.inner)
    }
    
    pub fn toggle_mark(self, flag: impl AsRef<OsStr>) -> io::Result<()> {
        mark::toggle(self.root, &self.inner, flag)
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
    
    pub fn delete(self) -> io::Result<()> {
        fs::remove_file(self.to_path())
    }
    
}
