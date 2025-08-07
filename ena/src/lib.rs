mod entry;
mod mark;

use std::{
    fs,
    io::{ self, Error, ErrorKind },
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
    
    pub fn walk<R: AsRef<Path>>(root: R) -> io::Result<Self> {
        Self::with_depth(root, INITIAL_DIRECTORY_DEPTH)
    }
    
    fn with_depth<C: AsRef<Path>>(current: C, depth: u8) -> io::Result<Self> {
        
        if depth > MAX_ALLOWED_DIRECTORY_DEPTH {
            return Err(Error::new(ErrorKind::InvalidInput, "Maximum directory depth exceeded"));
        }
        
        let current = current.as_ref();
        
        if current.metadata()?.is_symlink() {
            return Err(Error::new(ErrorKind::InvalidInput, "Symlinks are not supported"));
        }
        
        Ok(Self {
            current: current.read_dir()?,
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
                match subdirectory.next() {
                    Some(entry) => return Some(entry),
                    None => self.subdirectory = None,
                }
            }
            
            // -------------------- current directory --------------------
            
            for entry in self.current.by_ref().flatten() {
                
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                
                // file
                
                if file_type.is_file() {
                    return Some(FilesEntry::new(entry.path(), self.depth));
                }
                
                // subdirectory
                
                if file_type.is_dir() && let Ok(subdirectory) = Self::with_depth(entry.path(), self.depth + 1) {
                    self.subdirectory = Some(Box::new(subdirectory));
                    continue 'outer;
                }
                
            }
            
            return None;
            
        }
        
    }
    
}
