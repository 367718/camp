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
    
    pub fn walk<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to query metadata for path '{}': {}", path.display(), error)))?;
        
        if ! metadata.is_dir() {
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid path: {}", path.display())));
        }
        
        Self::with_depth(path, INITIAL_DIRECTORY_DEPTH)
    }
    
    fn with_depth(path: &Path, depth: u8) -> io::Result<Self> {
        if depth > MAX_ALLOWED_DIRECTORY_DEPTH {
            return Err(Error::new(ErrorKind::InvalidInput, "Maximum directory depth exceeded"));
        }
        
        Ok(Self {
            current: path.read_dir()?,
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
                
                // does not traverse symlinks
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                
                // file
                
                if file_type.is_file() {
                    return Some(FilesEntry::new(entry.path(), self.depth));
                }
                
                // subdirectory
                
                if file_type.is_dir() && let Ok(subdirectory) = Self::with_depth(&entry.path(), self.depth + 1) {
                    self.subdirectory = Some(Box::new(subdirectory));
                    continue 'outer;
                }
                
            }
            
            return None;
            
        }
        
    }
    
}
