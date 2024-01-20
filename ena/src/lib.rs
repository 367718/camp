mod entry;
mod mark;

use std::{
    error::Error,
    fs,
    path::Path,
};

pub use entry::FilesEntry;

pub struct Files {
    directory: fs::ReadDir,
    subdirectory: Option<Box<Files>>,
}

impl Files {
    
    pub fn new(root: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            directory: root.read_dir()?,
            subdirectory: None,
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
            
            // -------------------- directory --------------------
            
            for entry in self.directory.by_ref().flatten() {
                
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                
                // symlink
                
                if file_type.is_symlink() {
                    continue;
                }
                
                // file
                
                if file_type.is_file() {
                    
                    let entry = entry.path()
                        .into_os_string()
                        .into_string()
                        .ok()
                        .map(FilesEntry::new);
                    
                    if entry.is_some() {
                        return entry;
                    }
                    
                    continue;
                    
                }
                
                // subdirectory
                
                if let Ok(subdirectory) = Files::new(&entry.path()) {
                    self.subdirectory = Some(Box::new(subdirectory));
                    continue 'outer;
                }
                
            }
            
            return None;
            
        }
        
    }
    
}
