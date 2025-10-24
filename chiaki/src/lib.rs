use std::{
    env,
    fs::{ self, File },
    io::{ self, Read, Write, Error, ErrorKind },
    mem,
    path::{ Path, PathBuf },
};

const MEM_SIZE: usize = mem::size_of::<u64>();
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
    pub value: u64,
}

impl List {
    
    // -------------------- constructors --------------------
    
    
    pub fn load(name: &str) -> io::Result<Self> {
        // -------------------- path --------------------
        
        // prevent directory traversal
        let clean = Path::new(name)
            .file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid list file name"))?;
        
        let mut path = env::current_dir()?;
        path.push(clean);
        path.set_extension("ck");
        
        // -------------------- metadata --------------------
        
        let metadata = fs::metadata(&path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to query metadata for list file '{}': {}", path.display(), error)))?;
        
        // -------------------- symlink --------------------
        
        if metadata.is_symlink() {
            return Err(Error::new(ErrorKind::InvalidInput, "Symlinks are not supported"));
        }
        
        // -------------------- file --------------------
        
        let file = File::open(&path)
            .map_err(|error| Error::new(error.kind(), format!("Failed to open list file '{}': {}", path.display(), error)))?;
        
        // -------------------- content --------------------
        
        let size = metadata.len().min(CONTENT_SIZE_LIMIT);
        
        let mut content = Vec::new();
        content.reserve_exact(usize::try_from(size).unwrap());
        
        let mut reader = file.take(size);
        reader.read_to_end(&mut content)
            .map_err(|error| Error::new(error.kind(), format!("Failed to read list file '{}': {}", path.display(), error)))?;
        
        Ok(Self {
            path,
            content,
        })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn iter(&self) -> ListIter<'_> {
        ListIter { content: &self.content }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn insert(&mut self, tag: &[u8], value: u64) -> io::Result<()> {
        if self.iter().any(|current| current.tag.eq_ignore_ascii_case(tag)) {
            return Err(Error::new(ErrorKind::AlreadyExists, "Tag in use"));
        }
        
        let capacity = self.content.len() + (tag.len() + MEM_SIZE * 2);
        let entries = Some(ListEntry { tag, value })
            .into_iter()
            .chain(self.iter());
        
        self.commit(Self::serialize(capacity, entries))
    }
    
    pub fn update(&mut self, tag: &[u8], value: u64) -> io::Result<()> {
        let position = self.iter().position(|current| current.tag.eq_ignore_ascii_case(tag))
            .ok_or(Error::new(ErrorKind::NotFound, "Tag not found"))?;
        
        let capacity = self.content.len();
        let entries = Some(ListEntry { tag, value })
            .into_iter()
            .chain(
                self.iter()
                    .enumerate()
                    .filter_map(|(current, entry)| (current != position).then_some(entry))
            );
        
        self.commit(Self::serialize(capacity, entries))
    }
    
    pub fn delete(&mut self, tag: &[u8]) -> io::Result<()> {
        let position = self.iter().position(|current| current.tag.eq_ignore_ascii_case(tag))
            .ok_or(Error::new(ErrorKind::NotFound, "Tag not found"))?;
        
        let capacity = self.content.len() - (tag.len() + MEM_SIZE * 2);
        let entries = self.iter()
            .enumerate()
            .filter_map(|(current, entry)| (current != position).then_some(entry));
        
        self.commit(Self::serialize(capacity, entries))
    }
    
    fn commit(&mut self, content: Vec<u8>) -> io::Result<()> {
        let mut file_name = self.path.file_name()
            .expect("Invalid list file path")
            .to_os_string();
        
        file_name.push(".tmp");
        
        let tmp_path = chikuwa::EphemeralPath::from(self.path.with_file_name(file_name));
        
        let mut file = File::options()
            .create_new(true)
            .write(true)
            .open(&tmp_path)?;
        
        file.write_all(&content)?;
        file.sync_data()?;
        
        // attempt to perform the update atomically
        fs::rename(&tmp_path, &self.path)?;
        
        // since the path no longer exists, do not attempt to remove it
        tmp_path.make_permanent();
        
        self.content = content;
        
        Ok(())
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn serialize<'c>(capacity: usize, entries: impl Iterator<Item = ListEntry<'c>>) -> Vec<u8> {
        let mut content = Vec::with_capacity(capacity);
        
        for entry in entries {
            content.extend_from_slice(&u64::try_from(entry.tag.len()).unwrap().to_le_bytes());
            content.extend_from_slice(entry.tag);
            content.extend_from_slice(&entry.value.to_le_bytes());
        }
        
        content
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
        // -------------------- tag size --------------------
        
        let (current, rest) = self.content.split_at_checked(MEM_SIZE)?;
        let tag_size = usize::try_from(u64::from_le_bytes(unsafe { current.try_into().unwrap_unchecked() }))
            .expect("Tag size exceeded the maximum value supported by the plataform");
        
        // -------------------- tag --------------------
        
        let (current, rest) = rest.split_at_checked(tag_size)?;
        let tag = current;
        
        // -------------------- value --------------------
        
        let (current, rest) = rest.split_at_checked(MEM_SIZE)?;
        let value = u64::from_le_bytes(unsafe { current.try_into().unwrap_unchecked() });
        
        self.content = rest;
        
        Some(ListEntry {
            tag,
            value,
        })
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod serialization_and_deserialization {
        
        use super::*;
        
        #[test]
        fn one_entry() {
            // setup
            
            let entries = [
                ListEntry {
                    tag: b"ftag",
                    value: 1,
                },
            ];
            
            let list = List {
                path: PathBuf::new(),
                content: List::serialize(0, entries.into_iter()),
            };
            
            // operation
            
            let mut output = list.iter();
            
            // control
            
            assert!(output.next() == Some(ListEntry {
                tag: b"ftag",
                value: 1,
            }));
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn two_entries() {
            // setup
            
            let entries = [
                ListEntry {
                    tag: b"ftag",
                    value: 1,
                },
                ListEntry {
                    tag: b"stag",
                    value: 2,
                },
            ];
            
            let list = List {
                path: PathBuf::new(),
                content: List::serialize(0, entries.into_iter()),
            };
            
            // operation
            
            let mut output = list.iter();
            
            // control
            
            assert!(output.next() == Some(ListEntry {
                tag: b"ftag",
                value: 1,
            }));
            
            assert!(output.next() == Some(ListEntry {
                tag: b"stag",
                value: 2,
            }));
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_entries() {
            // setup
            
            let entries = [];
            
            let list = List {
                path: PathBuf::new(),
                content: List::serialize(0, entries.into_iter()),
            };
            
            // operation
            
            let mut output = list.iter();
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
}
