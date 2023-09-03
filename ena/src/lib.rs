mod entry;
mod marker;

use std::{
    fs,
    path::PathBuf,
};

pub use entry::FilesEntry;

pub struct Files {
    entries: Vec<FilesEntryKind>,
}

enum FilesEntryKind {
    File(PathBuf),
    Directory(PathBuf),
}

const ENTRIES_INITIAL_CAPACITY: usize = 50;

impl Files {
    
    pub fn new(initial: PathBuf) -> Self {
        let mut entries = Vec::with_capacity(ENTRIES_INITIAL_CAPACITY);
        
        // do not follow symlinks when asking for metadata
        if let Ok(file_type) = fs::symlink_metadata(&initial).map(|metadata| metadata.file_type()) {
            
            // file, dir and symlink tests are mutually exclusive
            if file_type.is_dir() {
                entries.push(FilesEntryKind::Directory(initial));
            } else if file_type.is_file() {
                entries.push(FilesEntryKind::File(initial));
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
                
                FilesEntryKind::File(path) => return Some(FilesEntry::new(path)),
                
                FilesEntryKind::Directory(path) => {
                    
                    let Ok(directory) = path.read_dir() else {
                        continue;
                    };
                    
                    for entry in directory.flatten() {
                        
                        let Ok(file_type) = entry.file_type() else {
                            continue;
                        };
                        
                        // file, dir and symlink tests are mutually exclusive
                        if file_type.is_file() {
                            self.entries.push(FilesEntryKind::File(entry.path()));
                        } else if file_type.is_dir() {
                            self.entries.push(FilesEntryKind::Directory(entry.path()));
                        }
                        
                    }
                    
                },
                
            }
        }
        
        None
    }
    
}
