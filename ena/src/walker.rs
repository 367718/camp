use std::{
    fs,
    path::PathBuf,
};

pub struct FilesWalker {
    entries: Vec<PathBuf>,
}

impl FilesWalker {
    
    pub fn new(initial: PathBuf) -> Self {
        Self {
            entries: Vec::from([initial]),
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
