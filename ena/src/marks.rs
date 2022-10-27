use std::{
    error::Error,
    ffi::{ OsStr, OsString },
    fs,
    path::Path,
};

use super::FilesEntry;

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
    
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::Updated,
            Self::Watched,
            Self::None,
        ].iter().copied()
    }
    
}

pub fn get(flag: &OsStr, path: &Path) -> FilesMark {
    fs::read(build_query(path, flag)).map_or(FilesMark::None, FilesMark::from)
}

pub fn set(flag: &OsStr, entry: &FilesEntry, mark: FilesMark) -> Result<bool, Box<dyn Error>> {
    if entry.mark == mark {
        return Ok(false);
    }
    
    fs::write(build_query(&entry.path, flag), mark.as_bytes())?;
    
    Ok(true)
}

fn build_query(path: &Path, flag: &OsStr) -> OsString {
    let str = path.as_os_str();
    let mut query = OsString::with_capacity(str.len() + 1 + flag.len());
    
    query.push(str);
    query.push(":");
    query.push(flag);
    
    query
}
