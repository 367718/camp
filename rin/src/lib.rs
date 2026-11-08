mod impls;

use std::{
    env,
    fs::OpenOptions,
    io::{ self, Read, Error, ErrorKind },
    os::windows::fs::OpenOptionsExt,
    sync::OnceLock,
};

use impls::ParamFromValue;

const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_SHARE_READ: u32 = 0x0000_0001;

const CONTENT_SIZE_LIMIT: u64 = 32 * 1024;

pub fn get<T: ParamFromValue<'static>>(key: &[u8]) -> io::Result<T> {
    let content = load_content();
    
    if let Some(value) = extract_value(content, key) {
        return T::param_from_value(value);
    }
    
    Err(Error::new(ErrorKind::NotFound, format!("Configuration key not found: '{}'", String::from_utf8_lossy(key))))
}

fn load_content() -> &'static [u8] {
    
    fn read_file() -> io::Result<Vec<u8>> {
        // -------------------- path --------------------
        
        let directory = env::current_dir()?;
        let executable_path = env::current_exe()?;
        
        let file_name = executable_path.file_name()
            .ok_or(Error::from(ErrorKind::InvalidFilename))?;
        
        let path = directory
            .join(file_name)
            .with_extension("rn");
        
        // -------------------- file --------------------
        
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT) // open the symbolic link or junction itself
            .share_mode(FILE_SHARE_READ) // prevent external writes and deletes
            .open(&path)?;
        
        // -------------------- metadata --------------------
        
        let metadata = file.metadata()
            .map_err(|error| Error::new(error.kind(), format!("Failed to query metadata for configuration file '{}': {}", path.display(), error)))?;
        
        if metadata.file_type().is_symlink() {
            return Err(Error::new(ErrorKind::InvalidInput, "Symlinks are not supported"));
        }
        
        if metadata.len() > CONTENT_SIZE_LIMIT {
            let message = format!("Configuration file '{}' exceeded maximum allowable size ({} > {})", path.display(), metadata.len(), CONTENT_SIZE_LIMIT);
            return Err(Error::new(ErrorKind::InvalidData, message));
        }
        
        // -------------------- content --------------------
        
        // allocation cannot exceed `CONTENT_SIZE_LIMIT`, as checked earlier
        let capacity = usize::try_from(metadata.len()).expect("Unsupported platform");
        let mut content = Vec::with_capacity(capacity);
        
        file.take(CONTENT_SIZE_LIMIT.saturating_add(1))
            .read_to_end(&mut content)
            .map_err(|error| Error::new(error.kind(), format!("Failed to read configuration file '{}': {}", path.display(), error)))?;
        
        // this should normally be unreachable, as external writes were denied and file size has already been checked before
        if content.len() as u64 > CONTENT_SIZE_LIMIT {
            let message = format!("Configuration file '{}' exceeded maximum allowable size ({} > {})", path.display(), content.len(), CONTENT_SIZE_LIMIT);
            return Err(Error::new(ErrorKind::InvalidData, message));
        }
        
        // -------------------- response --------------------
        
        Ok(content)
    }
    
    static CONTENT: OnceLock<Vec<u8>> = OnceLock::new();
    
    // TODO: replace with OnceLock::get_or_try_init in the future (https://github.com/rust-lang/rust/issues/109737)
    CONTENT.get_or_init(|| read_file().unwrap_or_else(|_| panic!("Failed to load configuration file")))
    
}

fn extract_value(content: &'static [u8], key: &[u8]) -> Option<&'static [u8]> {
    let range = chikuwa::delimited_range(content, key, b"\r\n")?;
    
    // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
    let mut components = content[range].splitn(2, |&curr| curr == b'=')
        .map(<[u8]>::trim_ascii_start);
    
    // to the right of '='
    components.nth(1)
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod extract_value {
        
        // single
        // multiple
        // case_mismatch
        // key_not_present
        // content_empty
        // key_empty
        // extra_whitespace
        // pair_malformed
        // no_linebreak
        // emoji
        
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
