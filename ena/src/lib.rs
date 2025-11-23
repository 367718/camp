mod entry;
mod mark;

use std::{
    fs,
    io::{ self, Error, ErrorKind },
    path::Path,
};

pub use entry::FilesEntry;

pub struct Files {
    current: fs::ReadDir,
    subdirectory: Option<Box<Files>>,
    max_depth: u64,
    current_depth: u64,
}

impl Files {
    
    pub fn walk<P: AsRef<Path>>(path: P, max_depth: u64) -> io::Result<Self> {
        let path = path.as_ref();
        
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to query metadata for path '{}': {}", path.display(), error)))?;
        
        if ! metadata.is_dir() {
            return Err(Error::new(ErrorKind::InvalidInput, format!("Invalid path: {}", path.display())));
        }
        
        Self::with_depth(path, max_depth, 1)
    }
    
    fn with_depth(path: &Path, max_depth: u64, current_depth: u64) -> io::Result<Self> {
        if current_depth > max_depth {
            return Err(Error::new(ErrorKind::InvalidInput, "Maximum directory depth exceeded"));
        }
        
        Ok(Self {
            current: path.read_dir()?,
            subdirectory: None,
            max_depth,
            current_depth,
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
                    return Some(FilesEntry::new(entry.path(), self.current_depth));
                }
                
                // subdirectory
                if file_type.is_dir() && let Ok(subdirectory) = Self::with_depth(&entry.path(), self.max_depth, self.current_depth + 1) {
                    self.subdirectory = Some(Box::new(subdirectory));
                    continue 'outer;
                }
                
            }
            
            return None;
            
        }
        
    }
    
}
