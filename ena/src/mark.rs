use std::{
    ffi::{ OsStr, OsString },
    fs,
    io::{ self, ErrorKind },
};

pub fn is_marked(path: impl AsRef<OsStr>, flag: impl AsRef<OsStr>) -> io::Result<bool> {
    fs::exists(build_query(path, flag))
}

pub fn toggle(path: impl AsRef<OsStr>, flag: impl AsRef<OsStr>) -> io::Result<()> {
    let stream = build_query(path, flag);
    
    // if mark existed, the job is already done
    // if mark didn't exist, create it
    match fs::remove_file(&stream) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => fs::write(&stream, ""),
        Err(error) => Err(error),
    }
}

fn build_query(path: impl AsRef<OsStr>, flag: impl AsRef<OsStr>) -> OsString {
    let path = path.as_ref();
    let flag = flag.as_ref();
    
    let mut query = OsString::with_capacity(path.len() + 1 + flag.len());
    
    query.push(path);
    query.push(":");
    query.push(flag);
    
    query
}
