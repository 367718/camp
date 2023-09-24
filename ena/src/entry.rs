use std::{
    borrow::Cow,
    error::Error,
    ffi::OsStr,
    fs,
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
    
    pub fn container(&self, root: &Path) -> Option<Cow<'_, str>> {
        self.path.strip_prefix(root)
            .ok()
            .and_then(Path::parent)
            .map(Path::to_string_lossy)
            .filter(|parent| ! parent.is_empty())
    }
    
    pub fn is_marked(&self, flag: &OsStr) -> bool {
        crate::marker::is_marked(&self.path, flag)
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn mark(&mut self, flag: &OsStr, value: bool) -> Result<(), Box<dyn Error>> {
        crate::marker::mark(&self.path, flag, value)?;
        Ok(())
    }
    
    pub fn move_to_folder(&mut self, root: &Path, folder: &OsStr) -> Result<(), Box<dyn Error>> {
        let destination = root.join(folder)
            .join(self.path.file_name().ok_or("Invalid file name")?);
        
        if Path::exists(&destination) {
            return Err(chikuwa::concat_str!("Destination already exists: '", &destination.to_string_lossy(), "'").into())
        }
        
        fs::rename(&self.path, &destination)?;
        
        self.path = destination;
        
        Ok(())
    }
    
    pub fn delete(&mut self) -> Result<(), Box<dyn Error>> {
        fs::remove_file(&self.path)?;
        Ok(())
    }
    
}
