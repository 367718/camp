mod entry;
mod mark;

use std::{
    fs,
    path::{ PathBuf, Component },
};

pub use entry::FilesEntry;

pub struct Files {
    entries: Vec<FilesystemEntry>,
}

enum FilesystemEntry {
    File(PathBuf),
    Directory(PathBuf),
}

const ENTRIES_INITIAL_CAPACITY: usize = 250;

impl Files {
    
    pub fn new(initial: PathBuf) -> Self {
        let mut entries = Vec::with_capacity(ENTRIES_INITIAL_CAPACITY);
        
        // prevent directory traversal attacks
        
        if ! initial.components().any(|component| component == Component::ParentDir) {
            
            // do not follow symlinks when asking for metadata
            if let Ok(file_type) = fs::symlink_metadata(&initial).map(|metadata| metadata.file_type()) {
                
                // file, dir and symlink tests are mutually exclusive
                if file_type.is_dir() {
                    entries.push(FilesystemEntry::Directory(initial));
                } else if file_type.is_file() {
                    entries.push(FilesystemEntry::File(initial));
                }
                
            }
            
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
            match current {
                
                FilesystemEntry::File(path) => {
                    
                    let entry = path.to_str().map(|path| FilesEntry::new(path.to_string()));
                    
                    if entry.is_some() {
                        return entry;
                    }
                    
                    continue;
                    
                },
                
                FilesystemEntry::Directory(path) => {
                    
                    let Ok(directory) = path.read_dir() else {
                        continue;
                    };
                    
                    for entry in directory.flatten() {
                        
                        let Ok(file_type) = entry.file_type() else {
                            continue;
                        };
                        
                        // file, dir and symlink tests are mutually exclusive
                        if file_type.is_file() {
                            self.entries.push(FilesystemEntry::File(entry.path()));
                        } else if file_type.is_dir() {
                            self.entries.push(FilesystemEntry::Directory(entry.path()));
                        }
                        
                    }
                    
                },
                
            }
            
        }
        
        None
    }
    
}
