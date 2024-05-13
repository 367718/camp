use std::{
    ffi::{ OsStr, OsString },
    fs,
    io,
    path::Path,
};

pub fn is_marked<P: AsRef<OsStr>, F: AsRef<OsStr>>(path: P, flag: F) -> io::Result<bool> {
    
    // possible future alternative: https://doc.rust-lang.org/std/fs/fn.try_exists.html
    
    Path::new(&build_query(path, flag)).try_exists()
    
}

pub fn toggle<P: AsRef<OsStr>, F: AsRef<OsStr>>(path: P, flag: F) -> io::Result<()> {
    let stream = build_query(path, flag);
    
    // possible future alternative: https://doc.rust-lang.org/std/fs/fn.try_exists.html
    
    if Path::new(&stream).try_exists()? {
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
