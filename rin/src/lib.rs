use std::{
    env,
    error::Error,
    fs,
    str,
};

pub struct Config {
    content: Vec<u8>,
}

impl Config {
    
    // ---------- constructors ----------
    
    
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let path = env::current_exe()?.with_extension("rn");
        
        fs::read(&path)
            .map(|content| Self { content })
            .map_err(|_| chikuwa::concat_str!("Load of configuration file failed: '", &path.to_string_lossy(), "'").into())
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get<'c>(&'c self, key: &[u8]) -> Result<&'c str, Box<dyn Error>> {
        if let Some(range) = chikuwa::tag_range(&self.content, key, b"\r\n") {
            if let [b' ', b'=', b' ', value @ ..] = &self.content[range] {
                return Ok(str::from_utf8(value)?);
            }
        }
        
        Err(chikuwa::concat_str!("Missing or invalid field: '", str::from_utf8(key)?, "'").into())
    }
    
}
