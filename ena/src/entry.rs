use std::{
    error::Error,
    ffi::OsStr,
    fs,
    path::{ MAIN_SEPARATOR, Path },
};

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FilesEntry {
    inner: String,
}

impl FilesEntry {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(inner: String) -> Self {
        Self { inner }
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn path(&self) -> &str {
        &self.inner
    }
    
    pub fn name(&self) -> &str {
        self.inner.rsplit_once(MAIN_SEPARATOR)
            .map_or(&self.inner, |(_, name)| name)
    }
    
    pub fn container(&self, root: &str) -> Option<&str> {
        if ! self.inner.starts_with(root) {
            return None;
        }
        
        self.inner.rfind(MAIN_SEPARATOR)
            .map(|separator| &self.inner[root.len()..=separator])
            .map(|container| container.strip_prefix(MAIN_SEPARATOR).unwrap_or(container))
            .filter(|container| ! container.is_empty())
    }
    
    pub fn value(&self, flag: &str) -> u8 {
        u8::from(! crate::mark::is_marked(&self.inner, flag))
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn mark(&mut self, flag: &str) -> Result<(), Box<dyn Error>> {
        if crate::mark::is_marked(&self.inner, flag) {
            crate::mark::remove(&self.inner, flag)?;
        } else {
            crate::mark::add(&self.inner, flag)?;
        }
        Ok(())
    }
    
    pub fn move_to_folder(self, root: &str, name: &str) -> Result<(), Box<dyn Error>> {
        let name = name.rsplit_once(MAIN_SEPARATOR)
            .map_or(name, |(_, name)| name);
        
        let folder = Path::new(root).join(name);
        
        if ! folder.exists() {
            fs::create_dir(&folder)?;
        }
        
        let destination = folder.join(self.name());
        
        if destination.exists() {
            return Err(chikuwa::concat_str!("Destination already exists: '", &destination.to_string_lossy(), "'").into())
        }
        
        fs::rename(self.inner, &destination)?;
        
        Ok(())
    }
    
    pub fn delete(self) -> Result<(), Box<dyn Error>> {
        fs::remove_file(self.inner)?;
        Ok(())
    }
    
}

impl AsRef<OsStr> for FilesEntry {
    
    fn as_ref(&self) -> &OsStr {
        OsStr::new(&self.inner)
    }
    
}
