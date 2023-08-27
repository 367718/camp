use std::{
    ffi::OsString,
    fs,
    io,
    path::Path,
};

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum FilesMark {
    Updated,
    Watched,
    None,
}

impl From<Vec<u8>> for FilesMark {
    
    fn from(bytes: Vec<u8>) -> Self {
        match bytes.as_slice() {
            [2] => Self::Updated,
            [1] => Self::Watched,
            _ => Self::None,
        }
    }
    
}

impl FilesMark {
    
    fn as_bytes(self) -> [u8; 1] {
        match self {
            Self::Updated => [2],
            Self::Watched => [1],
            Self::None => [0],
        }
    }
    
    pub fn as_class(self) -> &'static str {
        match self {
            Self::Updated => "updated",
            Self::Watched => "watched",
            Self::None => "new",
        }
    }
    
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::Updated,
            Self::Watched,
            Self::None,
        ].iter().copied()
    }
    
}

pub fn get(path: &Path, flag: &str) -> FilesMark {
    fs::read(build_query(path, flag)).map_or(FilesMark::None, FilesMark::from)
}

pub fn set(path: &Path, flag: &str, mark: FilesMark) -> io::Result<()> {
    fs::write(build_query(path, flag), mark.as_bytes())
}

fn build_query(path: &Path, flag: &str) -> OsString {
    let os_str = path.as_os_str();
    let mut query = OsString::with_capacity(os_str.len() + 1 + flag.len());
    
    query.push(os_str);
    query.push(":");
    query.push(flag);
    
    query
}
