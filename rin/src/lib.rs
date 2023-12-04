use std::{
    env,
    error::Error,
    fs,
    str,
    sync::OnceLock,
};

pub fn get(key: &[u8]) -> Result<&'static str, Box<dyn Error>> {
    let content = load();
    
    if let Some(range) = chikuwa::tag_range(content, key, b"\r\n") {
        if let [b' ', b'=', b' ', value @ ..] = &content[range] {
            return Ok(str::from_utf8(value)?);
        }
    }
    
    Err(chikuwa::concat_str!("Missing or invalid field: '", &String::from_utf8_lossy(key), "'").into())
}

fn load() -> &'static [u8] {
    static CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
    CONTENT.get_or_init(|| {
        
        let path = env::current_exe()
            .expect("Failed to get executable name")
            .with_extension("rn");
        
        fs::read(&path).expect(&chikuwa::concat_str!("Load of config file located at '", &path.to_string_lossy(), "' failed"))
        
    })
}
