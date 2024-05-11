use std::{
    ffi::OsStr,
    fs,
    ops::Deref,
    path::{ Path, PathBuf },
};

pub struct EphemeralPath {
    inner: PathBuf,
    permanent: bool,
}

impl From<PathBuf> for EphemeralPath {
    
    fn from(value: PathBuf) -> Self {
        Self {
            inner: value,
            permanent: false,
        }
    }
    
}

impl Deref for EphemeralPath {
    
    type Target = Path;
    
    fn deref(&self) -> &Path {
        &self.inner
    }
    
}

impl AsRef<Path> for EphemeralPath {
    
    fn as_ref(&self) -> &Path {
        &self.inner
    }
    
}

impl AsRef<OsStr> for EphemeralPath {
    
    fn as_ref(&self) -> &OsStr {
        self.inner.as_os_str()
    }
    
}

impl Drop for EphemeralPath {
    
    fn drop(&mut self) {
        
        if self.permanent {
            return;
        }
        
        // symlinks will be skipped
        
        if self.inner.is_file() {
            fs::remove_file(&self.inner).ok();
        } else if self.inner.is_dir() {
            fs::remove_dir_all(&self.inner).ok();
        }
        
    }
    
}

impl EphemeralPath {
    
    pub fn make_permanent(mut self) {
        self.permanent = true;
    }
    
}
