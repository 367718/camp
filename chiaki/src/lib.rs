mod serdes;

use std::{
    env,
    fs::{ self, File },
    io::{ self, Write, BufWriter, Error, ErrorKind },
    path::{ Path, PathBuf },
};

const TAG_SIZE_LIMIT: usize = u16::MAX as usize;
const CONTENT_SIZE_LIMIT: u64 = 1024 * 512;

pub struct List {
    path: PathBuf,
    content: Vec<u8>,
}

pub struct ListIter<'c> {
    content: &'c [u8],
}

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ListEntry<'c> {
    pub tag: &'c [u8],
    pub value: u16,
}

impl List {
    
    // -------------------- constructors --------------------
    
    
    pub fn load(name: &str) -> io::Result<Self> {
        // -------------------- path --------------------
        
        // prevent directory traversal
        let file_name = Path::new(name)
            .file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid list file name"))?;
        
        let file_path = env::current_dir()?
            .join(file_name)
            .with_extension("ck");
        
        // -------------------- metadata --------------------
        
        let metadata = fs::metadata(&file_path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to query metadata for list file '{}': {}", file_path.display(), error)))?;
        
        // -------------------- symlink --------------------
        
        if metadata.is_symlink() {
            return Err(Error::new(ErrorKind::InvalidInput, "Symlinks are not supported"));
        }
        
        // -------------------- file --------------------
        
        let file = File::open(&file_path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to open list file '{}': {}", file_path.display(), error)))?;
        
        // -------------------- content --------------------
        
        let size = metadata.len().min(CONTENT_SIZE_LIMIT);
        
        let mut reader = chikuwa::LimitedReader::new(file, size)?;
        let mut content = Vec::with_capacity(usize::try_from(size).expect("Unsupported platform"));
        
        io::copy(&mut reader, &mut content)
            .map_err(|error| Error::new(error.kind(), format!("Failed to read list file '{}': {}", file_path.display(), error)))?;
        
        Ok(Self {
            path: file_path,
            content,
        })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn iter(&self) -> ListIter<'_> {
        ListIter { content: &self.content }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn set(self, tag: &[u8], value: u16) -> io::Result<()> {
        if tag.len() > TAG_SIZE_LIMIT {
            return Err(io::Error::new(ErrorKind::InvalidInput, "Tag size exceeded the maximum value supported"));
        }
        
        let entries = self.iter()
            .filter(|entry| entry.tag != tag)
            .chain(Some(ListEntry { tag, value }));
        
        Self::commit(&self.path, entries)
    }
    
    pub fn delete(self, tag: &[u8]) -> io::Result<()> {
        let entries = self.iter()
            .filter(|entry| entry.tag != tag);
        
        Self::commit(&self.path, entries)
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn commit<'c>(list_path: &Path, entries: impl Iterator<Item = ListEntry<'c>>) -> io::Result<()> {
        // serialize new content to temp file
        
        let mut temp_name = list_path.file_name()
            .expect("Invalid list file path")
            .to_os_string();
        
        temp_name.push(".tmp");
        
        let temp_path = chikuwa::EphemeralPath::from(list_path.with_file_name(temp_name));
        
        let temp_file = File::options()
            .create_new(true)
            .write(true)
            .open(&temp_path)?;
        
        let mut writer = BufWriter::new(temp_file);
        
        for entry in entries {
            serdes::serialize(&mut writer, &entry)?;
        }
        
        writer.flush()?;
        
        // attempt to update list file atomically
        
        fs::rename(&temp_path, list_path)?;
        
        // temp file should no longer exist
        
        temp_path.make_permanent();
        
        Ok(())
    }
    
}

impl <'c>IntoIterator for &'c List {
    
    type IntoIter = ListIter<'c>;
    type Item = ListEntry<'c>;
    
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
    
}

impl <'c>Iterator for ListIter<'c> {
    
    type Item = ListEntry<'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let (entry, rest) = serdes::deserialize(self.content)?;
        self.content = rest;
        
        Some(entry)
    }
    
}
