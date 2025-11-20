use std::{
    ffi::OsStr,
    fs,
    io::{ self, Error, ErrorKind },
    path::{ Path, PathBuf },
};

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FilesEntry {
    inner: PathBuf,
    depth: u64,
}

impl FilesEntry {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(inner: PathBuf, depth: u64) -> Self {
        Self { inner, depth }
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn path(&self) -> &Path {
        &self.inner
    }
    
    pub fn relative(&self) -> &Path {
        self.inner.strip_prefix(self.root())
            .expect("Discrepancy between full path and root")
    }
    
    pub fn container(&self) -> &OsStr {
        self.relative()
            .parent()
            .map_or_else(|| OsStr::new(""), Path::as_os_str)
    }
    
    pub fn file_name(&self) -> &OsStr {
        self.inner.file_name()
            .unwrap_or_else(|| OsStr::new(""))
    }
    
    pub fn is_marked<F: AsRef<OsStr>>(&self, flag: F) -> io::Result<bool> {
        crate::mark::is_marked(&self.inner, flag)
    }
    
    fn root(&self) -> &Path {
        self.inner.ancestors()
            .nth(usize::try_from(self.depth).expect("Unsupported platform"))
            .expect("Depth exceeded full path")
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn toggle_mark<F: AsRef<OsStr>>(self, flag: F) -> io::Result<()> {
        crate::mark::toggle(self.inner, flag)
    }
    
    pub fn move_to_folder<F: AsRef<Path>>(self, folder: F) -> io::Result<()> {
        let folder = folder.as_ref();
        
        // disallow the creation of additional directories
        let foldername = folder.file_name().unwrap_or(folder.as_os_str());
        let filename = self.file_name();
        
        let directory = self.root().join(foldername);
        let destination = directory.join(filename);
        
        if directory.try_exists()? {
            if destination.try_exists()? {
                return Err(Error::from(ErrorKind::AlreadyExists));
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
