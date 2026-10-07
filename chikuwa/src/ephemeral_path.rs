use std::{
    fs,
    path::{ Path, PathBuf },
};

pub struct EphemeralPath {
    inner: PathBuf,
    permanent: bool,
}

impl From<PathBuf> for EphemeralPath {
    
    fn from(content: PathBuf) -> Self {
        Self {
            inner: content,
            permanent: false,
        }
    }
    
}

impl AsRef<Path> for EphemeralPath {
    
    fn as_ref(&self) -> &Path {
        self.inner.as_path()
    }
    
}

impl Drop for EphemeralPath {
    
    fn drop(&mut self) {
        
        if self.permanent {
            return;
        }
        
        // symlinks will not be deleted
        if let Ok(metadata) = self.inner.symlink_metadata() {
            
            if metadata.is_file() {
                fs::remove_file(&self.inner).ok();
            } else if metadata.is_dir() {
                fs::remove_dir_all(&self.inner).ok();
            }
            
        }
        
    }
    
}

impl EphemeralPath {
    
    pub fn make_permanent(mut self) {
        self.permanent = true;
    }
    
}
