use std::{
    fs::{ ReadDir, DirEntry },
    io,
    path::Path,
};

pub struct Directory {
    inner: ReadDir,
    depth: u64,
}

impl Directory {
    
    pub fn new(path: &Path, depth: u64) -> io::Result<Self> {
        Ok(Self {
            inner: path.read_dir()?,
            depth,
        })
    }
    
    pub fn depth(&self) -> u64 {
        self.depth
    }
    
}


impl Iterator for Directory {
    
    type Item = DirEntry;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
            .and_then(Result::ok)
    }
    
}
