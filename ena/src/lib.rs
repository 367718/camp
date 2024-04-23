mod entry;
mod mark;

use std::{
    fs,
    io::{ self, Error },
    path::Path,
};

pub use entry::FilesEntry;

const INITIAL_DIRECTORY_DEPTH: u8 = 1;
const MAX_ALLOWED_DIRECTORY_DEPTH: u8 = 5;

pub struct Files {
    current: fs::ReadDir,
    subdirectory: Option<Box<Files>>,
    depth: u8,
}

impl Files {
    
    pub fn new<R: AsRef<Path>>(root: R) -> io::Result<Self> {
        Self::with_depth(root, INITIAL_DIRECTORY_DEPTH)
    }
    
    fn with_depth<C: AsRef<Path>>(current: C, depth: u8) -> io::Result<Self> {
        
        if depth > MAX_ALLOWED_DIRECTORY_DEPTH {
            return Err(Error::other("Maximum directory depth exceeded"));
        }
        
        Ok(Self {
            current: current.as_ref().read_dir()?,
            subdirectory: None,
            depth,
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
            
            for entry in self.current.by_ref().flatten() {
                
                let path = entry.path();
                
                // file
                
                if path.is_file() {
                    return Some(FilesEntry::new(path, self.depth));
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
