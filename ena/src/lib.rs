mod readdir;
mod entry;
mod mark;
mod sys;

use std::{
    ffi::OsString,
    io,
    os::windows::ffi::{ OsStrExt, OsStringExt },
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
            root: Self::to_verbatim_absolute(path.as_ref())?,
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
    
    
    // -------------------- helpers --------------------
    
    
    fn to_verbatim_absolute(path: &Path) -> io::Result<PathBuf> {
        // normalize provided path to a verbatim (`\\?\`) absolute path
        // this bypasses Windows' 260-char `MAX_PATH` limit
        
        // TODO: replace with <path>.absolute in the future (https://github.com/rust-lang/rust/issues/153328)
        let absolute = path::absolute(path)?;
        let absolute_as_os_str = absolute.as_os_str();
        let bytes = absolute_as_os_str.as_encoded_bytes();
        
        // preserve paths that already opt into the verbatim (`\\?\`) or Win32 device (`\\.\`) namespace
        if bytes.starts_with(br"\\?\") || bytes.starts_with(br"\\.\") {
            return Ok(absolute);
        }
        
        let out = if bytes.starts_with(br"\\") {
            
            // network shares (`\\server\share`) require `\\?\UNC\server\share`, not `\\?\\\server\share`
            let rest = absolute_as_os_str
                .encode_wide()
                .skip(2)
                .collect::<Vec<u16>>();
            
            let mut out = OsString::with_capacity(r"\\?\UNC\".len() + bytes.len() - r"\\".len());
            out.push(r"\\?\UNC\");
            out.push(OsString::from_wide(&rest));
            
            out
            
        } else {
            
            let mut out = OsString::with_capacity(r"\\?\".len() + absolute_as_os_str.len());
            out.push(r"\\?\");
            out.push(absolute_as_os_str);
            
            out
            
        };
        
        Ok(PathBuf::from(out))
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
