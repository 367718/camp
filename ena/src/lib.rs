mod entry;
mod mark;

use std::{
    error::Error,
    fs,
    path::Path,
};

pub use entry::FilesEntry;

const INITIAL_DIRECTORY_DEPTH: u8 = 1;
const MAX_ALLOWED_DIRECTORY_DEPTH: u8 = 5;

pub struct Files {
    depth: u8,
    current: fs::ReadDir,
    subdirectory: Option<Box<Files>>,
}

impl Files {
    
    pub fn new(root: &Path) -> Result<Self, Box<dyn Error>> {
        Self::with_depth(root, INITIAL_DIRECTORY_DEPTH)
    }
    
    fn with_depth(root: &Path, depth: u8) -> Result<Self, Box<dyn Error>> {
        
        if depth > MAX_ALLOWED_DIRECTORY_DEPTH {
            return Err("Maximum directory depth exceeded".into());
        }
        
        Ok(Self {
            depth,
            current: root.read_dir()?,
            subdirectory: None,
        })
        
    }
    
}

impl Iterator for Files {
    
    type Item = FilesEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        
        'outer: loop {
            
            // -------------------- subdirectory --------------------
            
            if let Some(subdirectory) = self.subdirectory.as_mut() {
                
                let entry = subdirectory.next();
                
                if entry.is_some() {
                    return entry;
                }
                
                self.subdirectory = None;
                
            }
            
            // -------------------- current directory --------------------
            
            'inner: for entry in self.current.by_ref().flatten() {
                
                let path = entry.path();
                
                // file
                
                if path.is_file() {
                    
                    let entry = path
                        .into_os_string()
                        .into_string()
                        .map(FilesEntry::new)
                        .ok();
                    
                    if entry.is_some() {
                        return entry;
                    }
                    
                    continue 'inner;
                    
                }
                
                // subdirectory
                
                if let Ok(subdirectory) = Files::with_depth(&path, self.depth + 1) {
                    self.subdirectory = Some(Box::new(subdirectory));
                    continue 'outer;
                }
                
            }
            
            return None;
            
        }
        
    }
    
}
