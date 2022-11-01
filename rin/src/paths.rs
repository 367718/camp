use std::{
    error::Error,
    fs::File,
    io::{ Write, BufWriter },
    path::{ Path, PathBuf },
};

use crate::Config;

pub struct Paths {
    files: PathBuf,
    downloads: PathBuf,
    pipe: PathBuf,
    database: PathBuf,
}

pub enum PathError {
    Encoding,
    Empty,
    Linebreak,
}

impl Paths {
    
    pub const DEFAULT_FILES: &'static str = "/example/files";
    pub const DEFAULT_DOWNLOADS: &'static str = "/example/downloads";
    pub const DEFAULT_PIPE: &'static str = "/example/pipe";
    pub const DEFAULT_DATABASE: &'static str = "/example/database";
    
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            files: PathBuf::from(Self::DEFAULT_FILES),
            downloads: PathBuf::from(Self::DEFAULT_DOWNLOADS),
            pipe: PathBuf::from(Self::DEFAULT_PIPE),
            database: PathBuf::from(Self::DEFAULT_DATABASE),
        }
    }
    
    pub fn serialize(&self, writer: &mut BufWriter<&File>) -> Result<(), Box<dyn Error>> {
        writeln!(writer, "paths.files = {}", self.files.to_string_lossy())?;
        writeln!(writer, "paths.downloads = {}", self.downloads.to_string_lossy())?;
        writeln!(writer, "paths.pipe = {}", self.pipe.to_string_lossy())?;
        writeln!(writer, "paths.database = {}", self.database.to_string_lossy())?;
        
        Ok(())
    }
    
    pub fn deserialize(content: &[u8]) -> Result<(Self, bool), Box<dyn Error>> {
        let mut corrected = false;
        
        let files = Config::get_value(content, b"paths.files")?;
        let downloads = Config::get_value(content, b"paths.downloads")?;
        let pipe = Config::get_value(content, b"paths.pipe")?;
        let database = Config::get_value(content, b"paths.database")?;
        
        let mut paths = Paths {
            files: PathBuf::from(files),
            downloads: PathBuf::from(downloads),
            pipe: PathBuf::from(pipe),
            database: PathBuf::from(database),
        };
        
        // files
        if Self::validate_path(&paths.files).is_err() {
            paths.files = PathBuf::from(Self::DEFAULT_FILES);
            corrected = true;
        }
        
        // downloads
        if Self::validate_path(&paths.downloads).is_err() {
            paths.downloads = PathBuf::from(Self::DEFAULT_DOWNLOADS);
            corrected = true;
        }
        
        // pipe
        if Self::validate_path(&paths.pipe).is_err() {
            paths.pipe = PathBuf::from(Self::DEFAULT_PIPE);
            corrected = true;
        }
        
        // database
        if Self::validate_path(&paths.database).is_err() {
            paths.database = PathBuf::from(Self::DEFAULT_DATABASE);
            corrected = true;
        }
        
        Ok((paths, corrected))
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn files(&self) -> &Path {
        &self.files
    }
    
    pub fn downloads(&self) -> &Path {
        &self.downloads
    }
    
    pub fn pipe(&self) -> &Path {
        &self.pipe
    }
    
    pub fn database(&self) -> &Path {
        &self.database
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn set_files<S: AsRef<Path>>(&mut self, files: S) -> Result<bool, Box<dyn Error>> {
        let files = files.as_ref();
        
        if self.files == files {
            return Ok(false);
        }
        
        Self::check_path(files, "Files")?;
        
        self.files = files.to_owned();
        
        Ok(true)
    }
    
    pub fn set_downloads<S: AsRef<Path>>(&mut self, downloads: S) -> Result<bool, Box<dyn Error>> {
        let downloads = downloads.as_ref();
        
        if self.downloads == downloads {
            return Ok(false);
        }
        
        Self::check_path(downloads, "Database")?;
        
        self.downloads = downloads.to_owned();
        
        Ok(true)
    }
    
    pub fn set_pipe<S: AsRef<Path>>(&mut self, pipe: S) -> Result<bool, Box<dyn Error>> {
        let pipe = pipe.as_ref();
        
        if self.pipe == pipe {
            return Ok(false);
        }
        
        Self::check_path(pipe, "Pipe")?;
        
        self.pipe = pipe.to_owned();
        
        Ok(true)
    }
    
    pub fn set_database<S: AsRef<Path>>(&mut self, database: S) -> Result<bool, Box<dyn Error>> {
        let database = database.as_ref();
        
        if self.database == database {
            return Ok(false);
        }
        
        Self::check_path(database, "Database")?;
        
        self.database = database.to_owned();
        
        Ok(true)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_path(path: &Path, field: &str) -> Result<(), Box<dyn Error>> {
        if let Err(error) = Self::validate_path(path) {
            match error {
                PathError::Encoding => return Err([field, ": invalid character detected"].concat().into()),
                PathError::Empty => return Err([field, ": cannot be empty"].concat().into()),
                PathError::Linebreak => return Err([field, ": cannot contain linebreaks"].concat().into()),
            }
        }
        
        Ok(())
    }
    
    pub fn validate_path(path: &Path) -> Result<(), PathError> {
        let str = path.to_str()
            .ok_or(PathError::Encoding)?;
        
        if str.is_empty() {
            return Err(PathError::Empty);
        }
        
        if str.chars().any(|char| char == '\n') {
            return Err(PathError::Linebreak);
        }
        
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    use std::{
        ffi::OsString,
        os::windows::prelude::*,
    };
    
    mod files {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let files = Path::new(Paths::DEFAULT_FILES);
            
            // operation
            
            let output = Paths::validate_path(files);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_files(Path::new("/testing/files"));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(paths.files(), Path::new("/testing/files"));
            
            assert_ne!(paths.files(), Path::new(Paths::DEFAULT_FILES));
        }
        
        #[test]
        fn invalid_non_utf8() {
            // setup
            
            let mut paths = Paths::new();
            
            // 0x0066 => f
            // 0x006f => o
            // 0xD800 => lone surrogate half (invalid in UTF-16)
            // 0x006f => o
            
            let source = [0x0066, 0x006f, 0xD800, 0x006f];
            let os_string = OsString::from_wide(&source[..]);
            
            // operation
            
            let output = paths.set_files(PathBuf::from(os_string));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.files(), Path::new(Paths::DEFAULT_FILES));
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_files(Path::new(""));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.files(), Path::new(Paths::DEFAULT_FILES));
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_files(Path::new("/example\n/files"));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.files(), Path::new(Paths::DEFAULT_FILES));
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_files(Path::new(Paths::DEFAULT_FILES));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(paths.files(), Path::new(Paths::DEFAULT_FILES));
        }
        
    }
    
    mod downloads {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let downloads = Path::new(Paths::DEFAULT_DOWNLOADS);
            
            // operation
            
            let output = Paths::validate_path(downloads);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_downloads(Path::new("/testing/downloads"));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(paths.downloads(), Path::new("/testing/downloads"));
            
            assert_ne!(paths.downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
        }
        
        #[test]
        fn invalid_non_utf8() {
            // setup
            
            let mut paths = Paths::new();
            
            // 0x0066 => f
            // 0x006f => o
            // 0xD800 => lone surrogate half (invalid in UTF-16)
            // 0x006f => o
            
            let source = [0x0066, 0x006f, 0xD800, 0x006f];
            let os_string = OsString::from_wide(&source[..]);
            
            // operation
            
            let output = paths.set_downloads(PathBuf::from(os_string));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_downloads(Path::new(""));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_downloads(Path::new("/example\n/downloads"));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_downloads(Path::new(Paths::DEFAULT_DOWNLOADS));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(paths.downloads(), Path::new(Paths::DEFAULT_DOWNLOADS));
        }
        
    }
    
    mod pipe {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let pipe = Path::new(Paths::DEFAULT_PIPE);
            
            // operation
            
            let output = Paths::validate_path(pipe);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_pipe(Path::new("/example/test"));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(paths.pipe(), Path::new("/example/test"));
            
            assert_ne!(paths.pipe(), Path::new(Paths::DEFAULT_PIPE));
        }
        
        #[test]
        fn invalid_non_utf8() {
            // setup
            
            let mut paths = Paths::new();
            
            // 0x0066 => f
            // 0x006f => o
            // 0xD800 => lone surrogate half (invalid in UTF-16)
            // 0x006f => o
            
            let source = [0x0066, 0x006f, 0xD800, 0x006f];
            let os_string = OsString::from_wide(&source[..]);
            
            // operation
            
            let output = paths.set_pipe(PathBuf::from(os_string));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.pipe(), Path::new(Paths::DEFAULT_PIPE));
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_pipe(Path::new(""));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.pipe(), Path::new(Paths::DEFAULT_PIPE));
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_pipe(Path::new("/example\n/pipe"));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.pipe(), Path::new(Paths::DEFAULT_PIPE));
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_pipe(Path::new(Paths::DEFAULT_PIPE));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(paths.pipe(), Path::new(Paths::DEFAULT_PIPE));
        }
        
    }
    
    mod database {
        
        use super::*;
        
        #[test]
        fn default() {
            // setup
            
            let database = Path::new(Paths::DEFAULT_DATABASE);
            
            // operation
            
            let output = Paths::validate_path(database);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn valid() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_database(Path::new("/testing/database"));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), true);
            
            assert_eq!(paths.database(), Path::new("/testing/database"));
            
            assert_ne!(paths.database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
        #[test]
        fn invalid_non_utf8() {
            // setup
            
            let mut paths = Paths::new();
            
            // 0x0066 => f
            // 0x006f => o
            // 0xD800 => lone surrogate half (invalid in UTF-16)
            // 0x006f => o
            
            let source = [0x0066, 0x006f, 0xD800, 0x006f];
            let os_string = OsString::from_wide(&source[..]);
            
            // operation
            
            let output = paths.set_database(PathBuf::from(os_string));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
        #[test]
        fn invalid_empty() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_database(Path::new(""));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
        #[test]
        fn invalid_linebreak() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_database(Path::new("/example\n/database"));
            
            // control
            
            assert!(output.is_err());
            
            assert_eq!(paths.database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut paths = Paths::new();
            
            // operation
            
            let output = paths.set_database(Path::new(Paths::DEFAULT_DATABASE));
            
            // control
            
            assert!(output.is_ok());
            
            assert_eq!(output.unwrap(), false);
            
            assert_eq!(paths.database(), Path::new(Paths::DEFAULT_DATABASE));
        }
        
    }
    
}
