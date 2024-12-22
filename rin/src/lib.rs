use std::{
    env,
    fs::File,
    io::{ self, Read, Error, ErrorKind },
    str,
    sync::OnceLock,
};

const CONTENT_SIZE_LIMIT: u64 = 32 * 1024;

pub fn get(key: &[u8]) -> io::Result<&'static str> {
    let content = content();
    
    if let Some(line) = chikuwa::subslice_range(content, key, b"\r\n") {
        if let [b' ', b'=', b' ', value @ ..] = &content[line] {
            return str::from_utf8(value)
                .map_err(|error| Error::new(ErrorKind::InvalidData, error));
        }
    }
    
    Err(Error::new(ErrorKind::NotFound, format!("Configuration key not found: '{}'", String::from_utf8_lossy(key))))
}

fn content() -> &'static [u8] {
    
    fn load() -> io::Result<Vec<u8>> {
        let app_path = env::current_exe()?;
        
        // prevent directory traversal
        let clean = app_path.file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid configuration file name"))?;
        
        let mut path = env::current_dir()?;
        path.push(clean);
        path.set_extension("rn");
        
        let file = File::open(&path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to open configuration file '{}': {}", path.to_string_lossy(), error)))?;
        
        let size = file.metadata()
            .map(|metadata| metadata.len().min(CONTENT_SIZE_LIMIT))
            .unwrap_or(0);
        
        let mut content = Vec::new();
        content.reserve_exact(usize::try_from(size).unwrap());
        
        let mut reader = file.take(size);
        reader.read_to_end(&mut content)
            .map_err(|error| Error::new(error.kind(), format!("Failed to read configuration file '{}': {}", path.to_string_lossy(), error)))?;
        
        Ok(content)
    }
    
    static CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
    
    // possible future alternative: https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_or_try_init
    
    CONTENT.get_or_init(|| load().unwrap_or_else(|_| panic!("Failed to load configuration file")))
    
}
