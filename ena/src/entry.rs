use std::{
    error::Error,
    fs,
    path::{ MAIN_SEPARATOR, Path },
};

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FilesEntry {
    path: String,
}

impl FilesEntry {
    
    // ---------- constructors ----------
    
    
    pub(crate) fn new(path: String) -> Self {
        Self { path }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn path(&self) -> &str {
        &self.path
    }
    
    pub fn name(&self) -> &str {
        match self.path.rsplit_once(MAIN_SEPARATOR) {
            Some((_, name)) => name,
            None => &self.path,
        }
    }
    
    pub fn container(&self, root: &str) -> Option<&str> {
        self.path.strip_prefix(root)?
            .rsplit_once(MAIN_SEPARATOR)
            .map(|(container, _)| container)
    }
    
    pub fn is_marked(&self, flag: &str) -> bool {
        crate::marker::is_marked(&self.path, flag)
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn mark(&mut self, flag: &str) -> Result<(), Box<dyn Error>> {
        crate::marker::mark(&self.path, flag, ! self.is_marked(flag))?;
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
        
        fs::rename(self.path, &destination)?;
        
        Ok(())
    }
    
    pub fn delete(self) -> Result<(), Box<dyn Error>> {
        fs::remove_file(self.path)?;
        Ok(())
    }
    
}
