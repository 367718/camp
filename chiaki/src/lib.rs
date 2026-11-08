mod serdes;

use std::{
    env,
    fs::{ self, File, OpenOptions },
    io::{ self, Read, BufWriter, Error, ErrorKind },
    os::windows::fs::OpenOptionsExt,
    path::{ Path, PathBuf },
};

pub struct List {
    path: PathBuf,
    content: Vec<u8>,
}

pub struct ListEntries<'c> {
    content: &'c [u8],
}

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct ListEntry<'c> {
    pub tag: &'c [u8],
    pub value: u16,
}

const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const FILE_SHARE_READ: u32 = 0x0000_0001;

const TAG_SIZE_LIMIT: usize = u16::MAX as usize;

impl List {
    
    // -------------------- constructors --------------------
    
    
    pub fn load(name: &str, max_size: u64) -> io::Result<Self> {
        // -------------------- path --------------------
        
        let file_name = Path::new(name);
        
        // guard against directory traversals
        if file_name.components().count() != 1 || file_name.file_name().is_none() || name.contains(':') {
            return Err(Error::new(ErrorKind::InvalidInput, "Invalid list file name"));
        }
        
        let path = env::current_dir()?
            .join(file_name)
            .with_added_extension("ck");
        
        // -------------------- file --------------------
        
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT) // open the symbolic link or junction itself
            .share_mode(FILE_SHARE_READ) // prevent external writes and deletes
            .open(&path)?;
        
        // -------------------- metadata --------------------
        
        let metadata = file.metadata()
            .map_err(|error| Error::new(error.kind(), format!("Failed to query metadata for list file '{}': {}", path.display(), error)))?;
        
        if metadata.file_type().is_symlink() {
            return Err(Error::new(ErrorKind::InvalidInput, "Symlinks are not supported"));
        }
        
        if metadata.len() > max_size {
            let message = format!("List file '{}' exceeded maximum allowable size ({} > {})", path.display(), metadata.len(), max_size);
            return Err(Error::new(ErrorKind::InvalidData, message));
        }
        
        // -------------------- content --------------------
        
        // allocation cannot exceed `max_size`, as checked earlier
        let capacity = usize::try_from(metadata.len()).expect("Unsupported platform");
        let mut content = Vec::with_capacity(capacity);
        
        file.take(max_size.saturating_add(1))
            .read_to_end(&mut content)
            .map_err(|error| Error::new(error.kind(), format!("Failed to read list file '{}': {}", path.display(), error)))?;
        
        // this should normally be unreachable, as external writes were denied and file size has already been checked before
        if content.len() as u64 > max_size {
            let message = format!("List file '{}' exceeded maximum allowable size ({} > {})", path.display(), content.len(), max_size);
            return Err(Error::new(ErrorKind::InvalidData, message));
        }
        
        // -------------------- response --------------------
        
        Ok(Self { path, content })
    }
    
    
    // -------------------- accessors --------------------
    
    
    #[allow(clippy::iter_without_into_iter)]
    pub fn iter(&self) -> ListEntries<'_> {
        ListEntries { content: &self.content }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn set(self, tag: &[u8], value: u16) -> io::Result<()> {
        if tag.len() > TAG_SIZE_LIMIT {
            return Err(io::Error::new(ErrorKind::InvalidInput, "Tag size exceeds the maximum value supported"));
        }
        
        if self.iter().any(|entry| entry.tag == tag && entry.value == value) {
            return Ok(());
        }
        
        let entries = self.iter()
            .filter(|entry| entry.tag != tag)
            .chain(Some(ListEntry { tag, value }));
        
        Self::commit(&self.path, entries)
    }
    
    pub fn delete(self, tag: &[u8]) -> io::Result<()> {
        if ! self.iter().any(|entry| entry.tag == tag) {
            return Ok(());
        }
        
        let entries = self.iter()
            .filter(|entry| entry.tag != tag);
        
        Self::commit(&self.path, entries)
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn commit<'c>(list_path: &Path, mut entries: impl Iterator<Item = ListEntry<'c>>) -> io::Result<()> {
        let temp_path = chikuwa::EphemeralPath::from(list_path.with_added_extension("tmp"));
        
        let temp_file = File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)?;
        
        let mut writer = BufWriter::new(temp_file);
        
        entries.try_for_each(|entry| serdes::serialize(&mut writer, &entry))?;
        
        // `flush` is called internally, which may fail
        let temp_file = writer.into_inner()?;
        
        // commit data to hardware
        temp_file.sync_data()?;
        
        // attempt to update list file atomically
        fs::rename(&temp_path, list_path)?;
        
        // temp file should no longer exist
        temp_path.make_permanent();
        
        Ok(())
    }
    
}

impl<'c> Iterator for ListEntries<'c> {
    
    type Item = ListEntry<'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let (entry, rest) = serdes::deserialize(self.content)?;
        self.content = rest;
        
        Some(entry)
    }
    
}
