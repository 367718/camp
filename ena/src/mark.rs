use std::{
    ffi::{ OsStr, OsString },
    fs,
    io,
};

pub fn is_marked<P: AsRef<OsStr>, F: AsRef<OsStr>>(path: P, flag: F) -> io::Result<bool> {
    fs::exists(build_query(path, flag))
}

pub fn toggle<P: AsRef<OsStr>, F: AsRef<OsStr>>(path: P, flag: F) -> io::Result<()> {
    let stream = build_query(path, flag);
    
    if fs::exists(&stream)? {
        fs::remove_file(&stream)
    } else {
        fs::write(&stream, [0])
    }
}

fn build_query<P: AsRef<OsStr>, F: AsRef<OsStr>>(path: P, flag: F) -> OsString {
    let path = path.as_ref();
    let flag = flag.as_ref();
    
    let mut query = OsString::with_capacity(path.len() + 1 + flag.len());
    
    query.push(path);
    query.push(":");
    query.push(flag);
    
    query
}
