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
    
    
    pub fn relative(&self, root: &str) -> &str {
        self.inner.strip_prefix(root)
            .map_or(&self.inner, |container| container.strip_prefix(MAIN_SEPARATOR).unwrap_or(container))
    }
    
    pub fn components(&self, root: &str) -> (&str, Option<&str>) {
        let relative = self.relative(root);
        
        relative.rfind(MAIN_SEPARATOR)
            .map(|index| relative.split_at(index + 1))
            .map_or((relative, None), |(directory, filename)| (filename, Some(directory)))
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
    
    pub fn move_to_folder(self, root: &str, foldername: &str) -> Result<(), Box<dyn Error>> {
        let foldername = foldername.rsplit(MAIN_SEPARATOR).next().unwrap_or(foldername);
        let filename = self.inner.rsplit(MAIN_SEPARATOR).next().ok_or("Invalid filename")?;
        let directory = Path::new(root).join(foldername);
        let destination = directory.join(filename);
        
        if directory.exists() {
            if destination.exists() {
                return Err(chikuwa::concat_str!("Destination already exists: '", &destination.to_string_lossy(), "'").into())
            }
        } else {
            fs::create_dir(&directory)?;
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
