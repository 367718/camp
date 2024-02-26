use std::{
    env,
    error::Error,
    fs,
    str,
    sync::OnceLock,
};

pub fn get(key: &[u8]) -> Result<&'static str, Box<dyn Error>> {
    let content = load();
    
    if let Some(range) = chikuwa::subslice_range(content, key, b"\r\n") {
        if let [b' ', b'=', b' ', value @ ..] = &content[range] {
            return Ok(str::from_utf8(value)?);
        }
    }
    
    Err(format!("Missing or invalid field: '{}'", &String::from_utf8_lossy(key)).into())
}

fn load() -> &'static [u8] {
    static CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
    CONTENT.get_or_init(|| {
        
        let path = env::current_exe()
            .expect("Failed to get executable name")
            .with_extension("rn");
        
        fs::read(&path).unwrap_or_else(|_| panic!("Load of configuration file located at '{}' failed", &path.to_string_lossy()))
        
    })
}
