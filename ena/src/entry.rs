use std::{
    ffi::OsStr,
    fs,
    io::{ self, Error, ErrorKind },
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
    
    
    pub fn path(&self) -> &Path {
        &self.inner
    }
    
    pub fn relative(&self) -> &Path {
        self.inner.strip_prefix(self.root)
            .expect("Discrepancy between full path and root")
    }
    
    pub fn is_marked<F: AsRef<OsStr>>(&self, flag: F) -> io::Result<bool> {
        crate::mark::is_marked(&self.inner, flag)
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn toggle_mark<F: AsRef<OsStr>>(self, flag: F) -> io::Result<()> {
        crate::mark::toggle(self.inner, flag)
    }
    
    pub fn move_to_folder<F: AsRef<Path>>(self, folder: F) -> io::Result<()> {
        // disallow the creation of additional directories
        let container_name = folder.as_ref().file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid folder name"))?;
        
        let container_path = self.root.join(container_name);
        
        let file_name = self.inner.file_name()
            .ok_or(Error::new(ErrorKind::InvalidFilename, "Invalid file name"))?;
        
        let file_path = container_path.join(file_name);
        
        if container_path.try_exists()? {
            if file_path.try_exists()? {
                return Err(Error::from(ErrorKind::AlreadyExists));
            }
        } else {
            fs::create_dir(&container_path)?;
        }
        
        fs::rename(self.inner, &file_path)
    }
    
    pub fn delete(self) -> io::Result<()> {
        fs::remove_file(self.inner)
    }
    
}

impl AsRef<Path> for FilesEntry<'_> {
    
    fn as_ref(&self) -> &Path {
        &self.inner
    }
    
}

impl AsRef<OsStr> for FilesEntry<'_> {
    
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.inner)
    }
    
}
