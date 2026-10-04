use std::{
    ffi::OsStr,
    fs,
    io,
    path::{ Path, PathBuf },
};

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
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
        self.inner.strip_prefix(self.root)
            .expect("Discrepancy between full path and root")
    }
    
    pub fn is_marked(&self, flag: impl AsRef<OsStr>) -> io::Result<bool> {
        crate::mark::is_marked(&self.inner, flag)
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn to_path(self) -> PathBuf {
        self.inner
    }
    
    pub fn toggle_mark(self, flag: impl AsRef<OsStr>) -> io::Result<()> {
        crate::mark::toggle(self.inner, flag)
    }
    
    pub fn move_to_folder(self, folder: impl AsRef<Path>) -> io::Result<()> {
        // if folder is empty or its 'file_name' cannot be extracted, entry will be moved to root
        let container_name = folder.as_ref()
            .file_name()
            .unwrap_or_default();
        
        let file_name = self.inner
            .file_name()
            .expect("Invalid entry file name");
        
        let target_path = self.root
            .join(container_name)
            .join(file_name);
        
        if self.inner == target_path {
            return Ok(());
        }
        
        fs::create_dir_all(target_path.parent().unwrap())?;
        
        let temp_path = chikuwa::EphemeralPath::from(target_path);
        
        // avoid toctou bug
        fs::hard_link(&self.inner, &temp_path)?;
        fs::remove_file(&self.inner)?;
        
        temp_path.make_permanent();
        
        Ok(())
    }
    
    pub fn delete(self) -> io::Result<()> {
        fs::remove_file(self.inner)
    }
    
}
