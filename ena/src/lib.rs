mod entry;
mod mark;

use std::{
    fs::ReadDir,
    path::{ Path, PathBuf }
};

pub use entry::FilesEntry;

pub struct Files {
    root: PathBuf,
    max_depth: u64,
}

pub struct FilesIter<'r> {
    inner: Vec<(ReadDir, u64)>,
    root: &'r Path,
    max_depth: u64,
}

impl Files {
    
    pub fn new<P: AsRef<Path>>(path: P, max_depth: u64) -> Self {
        Self {
            root: path.as_ref().to_path_buf(),
            max_depth,
        }
    }
    
    pub fn iter(&self) -> FilesIter<'_> {
        let capacity = usize::try_from(self.max_depth).expect("Unsupported platform");
        let mut directories = Vec::with_capacity(capacity);
        
        if let Ok(initial) = self.root.read_dir() {
            // the "ReadDir" struct, in Windows, holds a file handle, so:
            // a) exhaustion is a possibility
            // b) the directory it points to cannot be modified while the handle is held
            // the "inner" vec length is not unbounded however, as it shouldn't hold more than "max_depth" entries at any given time
            directories.push((initial, 1));
        }
        
        FilesIter {
            inner: directories,
            root: &self.root,
            max_depth: self.max_depth,
        }
    }
    
}

impl<'r> IntoIterator for &'r Files {
    
    type IntoIter = FilesIter<'r>;
    type Item = FilesEntry<'r>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
    
}

impl<'r> Iterator for FilesIter<'r> {
    
    type Item = FilesEntry<'r>;
    
    fn next(&mut self) -> Option<Self::Item> {
        
        while let Some((current_dir, current_depth)) = self.inner.last_mut() {
            
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
                let root = self.root;
                
                return Some(FilesEntry::new(path, root));
                
            }
            
            // -------------------- subdirectory --------------------
            
            if file_type.is_dir() && *current_depth < self.max_depth {
                
                let path = dir_entry.path();
                let depth = *current_depth + 1;
                
                if let Ok(subdirectory) = path.read_dir() {
                    self.inner.push((subdirectory, depth));
                }
                
            }
            
        }
        
        None
        
    }
    
}
