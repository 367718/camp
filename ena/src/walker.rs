use std::path::PathBuf;

const ENTRIES_INITIAL_CAPACITY: usize = 100;

pub struct FilesWalker {
    entries: Vec<EntryKind>,
}

enum EntryKind {
    File(PathBuf),
    Directory(PathBuf),
}

impl FilesWalker {
    
    pub fn new(initial: PathBuf) -> Self {
        let mut entries = Vec::with_capacity(ENTRIES_INITIAL_CAPACITY);
        
        if let Ok(file_type) = initial.metadata().map(|metadata| metadata.file_type()) {
            
            if file_type.is_dir() {
                entries.push(EntryKind::Directory(initial));
            } else if file_type.is_file() {
                entries.push(EntryKind::File(initial));
            }
            
        }
        
        Self {
            entries,
        }
    }
    
}

impl Iterator for FilesWalker {
    
    type Item = PathBuf;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.entries.pop() {
            match current {
                
                EntryKind::File(path) => return Some(path),
                
                EntryKind::Directory(path) => if let Ok(directory) = path.read_dir() {
                    for entry in directory.flatten() {
                        
                        if let Ok(file_type) = entry.file_type() {
                            
                            if file_type.is_file() {
                                self.entries.push(EntryKind::File(entry.path()));
                            } else if file_type.is_dir() {
                                self.entries.push(EntryKind::Directory(entry.path()));
                            }
                            
                        }
                        
                    }
                },
                
            }
        }
        
        None
    }
    
}
