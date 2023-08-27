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
        let path = env::args_os().skip_while(|arg| arg != "--config")
            .nth(1).ok_or("Missing config path argument")?;
        
        let content = fs::read(path)?;
        
        Ok(Self { content })
    }
    
    
    // ---------- helpers ----------
    
    
    pub fn get<'c>(&'c self, key: &[u8]) -> Result<&'c str, Box<dyn Error>> {
        if let Some(range) = chikuwa::tag_range(&self.content, key, b"\r\n") {
            if let [b' ', b'=', b' ', value @ ..] = &self.content[range] {
                return Ok(str::from_utf8(value)?);
            }
        }
        
        Err(chikuwa::concat_str!("Missing or invalid field: ", str::from_utf8(key)?).into())
    }
    
}
