use std::{
    env,
    fs,
    io::{ self, Error },
    str,
    sync::OnceLock,
};

pub fn get(key: &[u8]) -> io::Result<&'static str> {
    let content = load();
    
    if let Some(range) = chikuwa::subslice_range(content, key, b"\r\n") {
        if let [b' ', b'=', b' ', value @ ..] = &content[range] {
            return str::from_utf8(value)
                .map_err(|error| Error::other(error.to_string()));
        }
    }
    
    Err(Error::other(format!("Missing or invalid field: '{}'", &String::from_utf8_lossy(key))))
}

fn load() -> &'static [u8] {
    static CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
    
    // possible future alternative: https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_or_try_init
    CONTENT.get_or_init(|| {
        
        let path = env::current_exe()
            .expect("Failed to get executable name")
            .with_extension("rn");
        
        fs::read(&path)
            .unwrap_or_else(|_| panic!("Load of configuration file located at '{}' failed", &path.to_string_lossy()))
        
    })
}
