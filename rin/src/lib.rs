use std::{
    env,
    fs::File,
    io::{ self, Read, Error, ErrorKind },
    sync::OnceLock,
};

const CONTENT_SIZE_LIMIT: u64 = 32 * 1024;

pub fn get(key: &[u8]) -> io::Result<&'static str> {
    let content = load_content();
    
    if let Some(value) = extract_value(content, key) {
        return str::from_utf8(value)
            .map_err(|error| Error::new(ErrorKind::InvalidData, error));
    }
    
    Err(Error::new(ErrorKind::NotFound, format!("Configuration key not found: '{}'", String::from_utf8_lossy(key))))
}

fn load_content() -> &'static [u8] {
    
    fn read_file() -> io::Result<Vec<u8>> {
        let directory = env::current_dir()?;
        let executable_path = env::current_exe()?;
        
        let file_name = executable_path.file_name()
            .ok_or(Error::from(ErrorKind::InvalidFilename))?;
        
        let mut file_path = directory.join(file_name);
        file_path.set_extension("rn");
        
        let file = File::open(file_path)?;
        let mut handle = file.take(CONTENT_SIZE_LIMIT);
        
        let mut content = Vec::new();
        handle.read_to_end(&mut content)?;
        
        Ok(content)
    }
    
    static CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
    
    // possible future alternative: https://doc.rust-lang.org/std/sync/struct.OnceLock.html#method.get_or_try_init
    
    CONTENT.get_or_init(|| read_file().unwrap_or_else(|_| panic!("Failed to load configuration file")))
    
}

fn extract_value(content: &'static [u8], key: &[u8]) -> Option<&'static [u8]> {
    let line = chikuwa::subslice_range(content, key, b"\r\n")?;
    let (_, value) = chikuwa::split_slice_once(&content[line], b"=");
    
    Some(value.trim_ascii_start())
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod extract_value {
        
        use super::*;
        
        #[test]
        fn single() {
            // setup
            
            let content = b"key=value\r\n";
            let key = b"key";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert_eq!(output, Some(b"value".as_slice()));
        }
        
        #[test]
        fn multiple() {
            // setup
            
            let content = b"fkey=fvalue\r\nskey=svalue\r\ntkey=yvalue\r\n";
            let key = b"skey";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert_eq!(output, Some(b"svalue".as_slice()));
        }
        
        #[test]
        fn case_mismatch() {
            // setup
            
            let content = b"key=value\r\n";
            let key = b"KEY";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert_eq!(output, Some(b"value".as_slice()));
        }
        
        #[test]
        fn key_not_present() {
            // setup
            
            let content = b"fkey=fvalue\r\nskey=svalue\r\ntkey=yvalue\r\n";
            let key = b"ekey";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn content_empty() {
            // setup
            
            let content = b"\r\n";
            let key = b"key";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn key_empty() {
            // setup
            
            let content = b"key=value\r\n";
            let key = b"";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn extra_whitespace() {
            // setup
            
            let content = b"key  =   value\r\n";
            let key = b"key";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert_eq!(output, Some(b"value".as_slice()));
        }
        
        #[test]
        fn pair_malformed() {
            // setup
            
            let content = b"key=\r\n";
            let key = b"key";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert_eq!(output, Some(b"".as_slice()));
        }
        
        #[test]
        fn no_linebreak() {
            // setup
            
            let content = b"key=value";
            let key = b"key";
            
            // operation
            
            let output = extract_value(content, key);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn emoji() {
            // setup
            
            let content = "key🔌=val🔌ue\r\n";
            let key = "key🔌";
            
            // operation
            
            let output = extract_value(content.as_bytes(), key.as_bytes());
            
            // control
            
            assert_eq!(output, Some("val🔌ue".as_bytes()));
        }
        
    }
    
}
