mod entry;
mod marker;

use std::{
    fs,
    path::{ PathBuf, Component },
};

pub use entry::FilesEntry;

pub struct Files {
    entries: Vec<PathBuf>,
}

const ENTRIES_INITIAL_CAPACITY: usize = 250;

impl Files {
    
    pub fn new(initial: PathBuf) -> Self {
        let mut entries = Vec::with_capacity(ENTRIES_INITIAL_CAPACITY);
        
        entries.push(initial);
        
        Self {
            entries,
        }
    }
    
}

impl Iterator for Files {
    
    type Item = FilesEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.entries.pop() {
            
            // prevent directory traversal attacks
            
            if current.components().any(|component| component == Component::ParentDir) {
                continue;
            }
            
            let Ok(file_type) = fs::symlink_metadata(&current).map(|metadata| metadata.file_type()) else {
                continue;
            };
            
            // file, dir and symlink tests are mutually exclusive
            
            if file_type.is_file() {
                if let Some(path) = current.to_str().map(ToString::to_string) {
                    return Some(FilesEntry::new(path));
                }
            }
            
            if file_type.is_dir() {
                
                let Ok(directory) = current.read_dir() else {
                    continue;
                };
                
                for entry in directory.flatten() {
                    self.entries.push(entry.path());
                }
                
            }
            
        }
        
        None
    }
    
}
