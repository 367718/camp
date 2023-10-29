use std::{
    ffi::OsString,
    fs,
    io,
    path::Path,
};

pub fn is_marked(path: &str, flag: &str) -> bool {
    Path::new(&build_query(path, flag)).exists()
}

pub fn add(path: &str, flag: &str) -> io::Result<()> {
    fs::write(build_query(path, flag), [0])
}

pub fn remove(path: &str, flag: &str) -> io::Result<()> {
    fs::remove_file(build_query(path, flag))
}

fn build_query(path: &str, flag: &str) -> OsString {
    let mut query = OsString::with_capacity(path.len() + 1 + flag.len());
    
    query.push(path);
    query.push(":");
    query.push(flag);
    
    query
}
