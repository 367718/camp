mod entry;
mod mark;

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
        
        // prevent directory traversal attacks
        
        if ! initial.components().any(|component| component == Component::ParentDir) {
            entries.push(initial);
        }
        
        Self {
            entries,
        }
    }
    
}

impl Iterator for Files {
    
    type Item = FilesEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.entries.pop() {
            
            let Ok(file_type) = fs::symlink_metadata(&current).map(|metadata| metadata.file_type()) else {
                continue;
            };
            
            if file_type.is_file() {
                
                let entry = current.to_str().map(|path| FilesEntry::new(path.to_string()));
                
                if entry.is_some() {
                    return entry;
                }
                
                continue;
                
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
