use std::{
    ffi::OsString,
    fs,
    io,
};

pub fn is_marked(path: &str, flag: &str) -> bool {
    fs::read(build_query(path, flag)).map_or(false, |mark| mark != [0])
}

pub fn mark(path: &str, flag: &str, value: bool) -> io::Result<()> {
    fs::write(build_query(path, flag), [u8::from(value)])
}

fn build_query(path: &str, flag: &str) -> OsString {
    let mut query = OsString::with_capacity(path.len() + 1 + flag.len());
    
    query.push(path);
    query.push(":");
    query.push(flag);
    
    query
}
