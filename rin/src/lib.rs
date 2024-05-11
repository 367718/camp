use std::{
    env,
    fs,
    io::{ self, Error, ErrorKind },
    str,
    sync::OnceLock,
};

pub fn get(key: &[u8]) -> io::Result<&'static str> {
    let contents = contents();
    
    if let Some(range) = chikuwa::subslice_range(contents, key, b"\r\n") {
        if let [b' ', b'=', b' ', value @ ..] = &contents[range] {
            
            return str::from_utf8(value)
                .map_err(|error| Error::new(ErrorKind::InvalidData, error));
            
        }
    }
    
    Err(Error::new(ErrorKind::NotFound, "Key not found"))
}

fn contents() -> &'static [u8] {
    
    fn load() -> io::Result<Vec<u8>> {
        fs::read(env::current_exe()?.with_extension("rn"))
    }
    
    static CONTENTS: OnceLock<Vec<u8>> = OnceLock::new();
    
    // possible future alternative: https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_or_try_init
    
    CONTENTS.get_or_init(|| load().unwrap())
    
}
