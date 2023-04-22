use std::{
    collections::hash_map::RandomState,
    env,
    ffi::{ OsStr, OsString },
    fs,
    hash::{ BuildHasher, Hasher },
    ops::Deref,
    path::{ Path, PathBuf },
};

pub struct EphemeralPath {
    inner: PathBuf,
    managed: bool,
}

pub struct EphemeralPathBuilder {
    base: Option<PathBuf>,
    suffix: Option<OsString>,
}

impl From<PathBuf> for EphemeralPath {
    
    fn from(value: PathBuf) -> Self {
        Self {
            inner: value,
            managed: true,
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
        if self.managed && self.inner.exists() {
            if self.inner.is_file() {
                fs::remove_file(&self.inner).ok();
            } else if self.inner.is_dir() {
                fs::remove_dir_all(&self.inner).ok();
            }
        }
    }
    
}

impl EphemeralPath {
    
    pub fn builder() -> EphemeralPathBuilder {
        EphemeralPathBuilder::new()
    }
    
    pub fn unmanage(mut self) {
        self.managed = false;
    }
    
}

impl Default for EphemeralPathBuilder {
    
    fn default() -> Self {
        Self::new()
    }
    
}

impl EphemeralPathBuilder {
    
    pub fn new() -> Self {
        Self {
            base: None,
            suffix: None,
        }
    }
    
    pub fn with_base<P: Into<PathBuf>>(mut self, base: P) -> Self {
        self.base = Some(base.into());
        self
    }
    
    pub fn with_suffix<S: Into<OsString>>(mut self, suffix: S) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
    
    pub fn build(self) -> EphemeralPath {
        let mut inner = self.base.unwrap_or_else(env::temp_dir);
        let suffix = self.suffix.unwrap_or_else(OsString::new);
        
        let start = env!("CARGO_PKG_NAME");
        let middle = RandomState::new()
            .build_hasher()
            .finish()
            .to_string();
        let end = RandomState::new()
            .build_hasher()
            .finish()
            .to_string();
        
        let base = crate::concat_str!(start, "-", &middle, "-", &end);
        
        let mut name = OsString::with_capacity(base.len() + suffix.len());
        
        name.push(base);
        name.push(suffix);
        
        inner.push(name);
        
        EphemeralPath::from(inner)
    }
    
}
