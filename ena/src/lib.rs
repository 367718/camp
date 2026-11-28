mod readdir;
mod entry;
mod sys;

use std::{
    io,
    path::{ self, Path, PathBuf },
};

use readdir::{ EntryKind, ReadDir };
pub use entry::FilesEntry;

pub struct Files {
    root: PathBuf,
    max_depth: usize,
}

pub struct FilesEntries<'r> {
    current: Option<(ReadDir, PathBuf, usize)>,
    pending: Vec<(PathBuf, usize)>,
    root: &'r Path,
    max_depth: usize,
}

impl Files {
    
    // -------------------- constructors --------------------
    
    
    pub fn new(path: impl AsRef<Path>, max_depth: u64) -> io::Result<Self> {
        Ok(Self {
            root: path::absolute(path)?,
            max_depth: usize::try_from(max_depth).expect("Unsupported platform"),
        })
    }
    
    
    // -------------------- accessors --------------------
    
    
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> FilesEntries<'_> {
        let current = if self.max_depth > 0 {
            ReadDir::open(&self.root)
                .map(|directory| (directory, PathBuf::new(), 1))
        } else {
            None
        };
        
        FilesEntries {
            current,
            pending: Vec::new(),
            root: &self.root,
            max_depth: self.max_depth,
        }
    }
    
}

impl<'r> Iterator for FilesEntries<'r> {
    
    type Item = FilesEntry<'r>;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            
            if let Some((readdir, relative_dir, depth)) = &mut self.current {
                for (file_name, kind) in readdir {
                    
                    if matches!(kind, EntryKind::File) {
                        return Some(FilesEntry::new(relative_dir.join(file_name), self.root));
                    }
                    
                    if *depth < self.max_depth {
                        self.pending.push((relative_dir.join(file_name), *depth + 1));
                    }
                    
                }
            }
            
            // release current readdir handle
            self.current = None;
            
            let (next_relative_dir, next_depth) = self.pending.pop()?;
            
            self.current = ReadDir::open(&self.root.join(&next_relative_dir))
                .map(|readdir| (readdir, next_relative_dir, next_depth));
            
        }
    }
    
}
