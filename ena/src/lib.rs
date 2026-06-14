mod entry;
mod mark;

use std::{
    fs::ReadDir,
    path::{ Path, PathBuf }
};

pub use entry::FilesEntry;

pub struct Files {
    root: PathBuf,
    max_depth: usize,
}

// the "ReadDir" struct holds a file handle in Windows, so:
// a) exhaustion is a possibility
// b) the directory it points to cannot be modified while the handle is held
pub struct FilesEntries<'r> {
    inner: Vec<ReadDir>,
    root: &'r Path,
    max_depth: usize,
}

impl Files {
    
    pub fn new<P: AsRef<Path>>(path: P, max_depth: u64) -> Self {
        Self {
            root: path.as_ref().to_path_buf(),
            max_depth: usize::try_from(max_depth).expect("Unsupported platform"),
        }
    }
    
    pub fn iter(&self) -> FilesEntries<'_> {
        let mut directories = Vec::with_capacity(self.max_depth);
        
        if let Ok(initial) = self.root.read_dir() {
            directories.push(initial);
        }
        
        FilesEntries {
            inner: directories,
            root: &self.root,
            max_depth: self.max_depth,
        }
    }
    
}

impl<'r> IntoIterator for &'r Files {
    
    type IntoIter = FilesEntries<'r>;
    type Item = FilesEntry<'r>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
    
}

impl<'r> Iterator for FilesEntries<'r> {
    
    type Item = FilesEntry<'r>;
    
    fn next(&mut self) -> Option<Self::Item> {
        
        while let Some(current_dir) = self.inner.last_mut() {
            
            let Some(dir_entry) = current_dir.next() else {
                self.inner.pop();
                continue;
            };
            
            let Ok(dir_entry) = dir_entry else {
                continue;
            };
            
            // does not traverse symlinks
            let Ok(file_type) = dir_entry.file_type() else {
                continue;
            };
            
            // -------------------- file --------------------
            
            if file_type.is_file() {
                
                let path = dir_entry.path();
                
                return Some(FilesEntry::new(path, self.root));
                
            }
            
            // -------------------- subdirectory --------------------
            
            // the root directory is considered "depth 1"
            if file_type.is_dir() && self.inner.len() < self.max_depth {
                
                let path = dir_entry.path();
                
                if let Ok(subdirectory) = path.read_dir() {
                    self.inner.push(subdirectory);
                }
                
            }
            
        }
        
        None
        
    }
    
}
