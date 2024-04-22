use std::{
    ffi::OsStr,
    fs,
    io::{ self, Error },
    path::{ Path, PathBuf },
};

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FilesEntry {
    inner: PathBuf,
}

impl FilesEntry {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(inner: PathBuf) -> Self {
        Self { inner }
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn relative<R: AsRef<Path>>(&self, root: R) -> &Path {
        self.inner.strip_prefix(root)
            .unwrap_or(&self.inner)
    }
    
    pub fn container<R: AsRef<Path>>(&self, root: R) -> &OsStr {
        self.relative(root)
            .parent()
            .map_or_else(|| OsStr::new(""), Path::as_os_str)
    }
    
    pub fn file_name(&self) -> &OsStr {
        self.inner.file_name()
            .unwrap_or_else(|| self.inner.as_os_str())
    }
    
    pub fn is_marked<F: AsRef<OsStr>>(&self, flag: F) -> bool {
        crate::mark::is_marked(&self.inner, flag)
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn toggle_mark<F: AsRef<OsStr>>(self, flag: F) -> io::Result<()> {
        crate::mark::toggle(self.inner, flag)
    }
    
    pub fn move_to_folder<R: AsRef<Path>, F: AsRef<Path>>(self, root: R, folder: F) -> io::Result<()> {
        let folder = folder.as_ref();
        
        // disallow the creation of additional directories
        let foldername = folder.file_name().unwrap_or(folder.as_os_str());
        let filename = self.file_name();
        
        let directory = root.as_ref().join(foldername);
        let destination = directory.join(filename);
        
        if directory.exists() {
            if destination.exists() {
                return Err(Error::other(format!("Destination already exists: '{}'", &destination.to_string_lossy())));
            }
        } else {
            fs::create_dir(&directory)?;
        }
        
        fs::rename(self.inner, &destination)
    }
    
    pub fn delete(self) -> io::Result<()> {
        fs::remove_file(self.inner)
    }
    
}

impl AsRef<Path> for FilesEntry {
    
    fn as_ref(&self) -> &Path {
        &self.inner
    }
    
}

impl AsRef<OsStr> for FilesEntry {
    
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.inner)
    }
    
}
