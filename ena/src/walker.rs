use std::{
    fs,
    path::PathBuf,
};

const ENTRIES_INITIAL_CAPACITY: usize = 100;

pub struct FilesWalker {
    entries: Vec<PathBuf>,
}

impl FilesWalker {
    
    pub fn new(initial: PathBuf) -> Self {
        let mut entries = Vec::with_capacity(ENTRIES_INITIAL_CAPACITY);
        entries.push(initial);
        
        Self {
            entries,
        }
    }
    
}

impl Iterator for FilesWalker {
    
    type Item = PathBuf;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.entries.pop() {
            
            let Ok(metadata) = current.metadata() else {
                continue;
            };
            
            if metadata.is_symlink() {
                continue;
            }
            
            if metadata.is_file() {
                return Some(current);
            }
            
            if let Ok(directory) = fs::read_dir(current) {
                for entry in directory.flatten() {
                    self.entries.push(entry.path());
                }
            }
            
        }
        
        None
    }
    
}
