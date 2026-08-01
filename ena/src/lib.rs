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

pub struct FilesEntries<'r> {
    current: Option<(ReadDir, usize)>,
    pending: Vec<(PathBuf, usize)>,
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
        let current = self.root.read_dir()
            .ok()
            .map(|directory| (directory, 1));
        
        FilesEntries {
            current,
            pending: Vec::new(),
            root: self.root.as_path(),
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
        loop {
            
            if let Some((directory, depth)) = &mut self.current {
                for entry in directory {
                    
                    let Ok(entry) = entry else {
                        continue;
                    };
                    
                    let Ok(file_type) = entry.file_type() else {
                        continue;
                    };
                    
                    // -------------------- file --------------------
                    
                    if file_type.is_file() {
                        return Some(FilesEntry::new(entry.path(), self.root));
                    }
                    
                    // -------------------- subdirectory --------------------
                    
                    if file_type.is_dir() && *depth < self.max_depth {
                        self.pending.push((entry.path(), *depth + 1));
                    }
                    
                }
            }
            
            let (next_path, next_depth) = self.pending.pop()?;
            
            self.current = next_path.read_dir()
                .ok()
                .map(|directory| (directory, next_depth));
            
        }
    }
    
}
