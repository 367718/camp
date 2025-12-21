mod directory;
mod entry;
mod mark;

use std::path::{ Path, PathBuf };

use directory::Directory;
pub use entry::FilesEntry;

pub struct Files {
    root: PathBuf,
    max_depth: u64,
}

pub struct FilesIter<'r> {
    inner: Vec<Directory>,
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
        let mut directories = Vec::new();
        
        if let Ok(initial) = Directory::new(&self.root, 1) {
            directories.push(initial);
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
        
        while let Some(current_dir) = self.inner.last_mut() {
            
            let Some(entry) = current_dir.next() else {
                self.inner.pop();
                continue;
            };
            
            // does not traverse symlinks
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            
            // -------------------- file --------------------
            
            if file_type.is_file() {
                
                let path = entry.path();
                let root = self.root;
                
                return Some(FilesEntry::new(path, root));
                
            }
            
            // -------------------- subdirectory --------------------
            
            if file_type.is_dir() && current_dir.depth() < self.max_depth {
                
                let path = entry.path();
                let depth = current_dir.depth() + 1;
                
                if let Ok(subdirectory) = Directory::new(&path, depth) {
                    self.inner.push(subdirectory);
                }
                
            }
            
        }
        
        None
        
    }
    
}
