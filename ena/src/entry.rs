use std::{
    borrow::Cow,
    error::Error,
    ffi::OsStr,
    path::{ Path, PathBuf },
};

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FilesEntry {
    path: PathBuf,
}

impl FilesEntry {
    
    // ---------- constructors ----------
    
    
    pub(crate) fn new(path: PathBuf) -> Self {
        Self { path }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn path(&self) -> Cow<'_, str> {
        self.path.to_string_lossy()
    }
    
    pub fn name(&self) -> Cow<'_, str> {
        self.path.file_name()
            .unwrap_or_else(|| OsStr::new(""))
            .to_string_lossy()
    }
    
    pub fn container(&self, root: &str) -> Option<Cow<'_, str>> {
        self.path.strip_prefix(root)
            .ok()
            .and_then(Path::parent)
            .map(Path::as_os_str)
            .filter(|parent| ! parent.is_empty())
            .map(OsStr::to_string_lossy)
    }
    
    pub fn is_marked(&self, flag: &str) -> bool {
        crate::marker::is_marked(&self.path, flag)
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn mark(&mut self, flag: &str, value: bool) -> Result<(), Box<dyn Error>> {
        crate::marker::mark(&self.path, flag, value)?;
        Ok(())
    }
    
}
