use std::{
    ffi::{ OsStr, OsString },
    fs,
    io,
    path::Path,
};

pub fn is_marked(path: &Path, flag: &OsStr) -> bool {
    fs::read(build_query(path, flag)).map_or(false, |mark| mark != [0])
}

pub fn mark(path: &Path, flag: &OsStr, value: bool) -> io::Result<()> {
    fs::write(build_query(path, flag), [u8::from(value)])
}

fn build_query(path: &Path, flag: &OsStr) -> OsString {
    let os_str = path.as_os_str();
    let mut query = OsString::with_capacity(os_str.len() + 1 + flag.len());
    
    query.push(os_str);
    query.push(":");
    query.push(flag);
    
    query
}
