use std::{
    env,
    fs,
    io::{ self, Error, ErrorKind },
    mem,
    path::{ Path, PathBuf },
    str,
};

const MEM_SIZE: usize = mem::size_of::<u64>();

pub struct List {
    path: PathBuf,
    content: Vec<u8>,
}

pub struct ListIter<'c> {
    content: &'c [u8],
}

pub struct ListEntry<'c> {
    pub tag: &'c [u8],
    pub value: u64,
}

impl List {
    
    // -------------------- constructors --------------------
    
    
    pub fn load(name: &str) -> io::Result<Self> {
        // prevent directory traversal
        let clean = Path::new(name)
            .file_name()
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid list file name"))?;
        
        let mut path = env::current_dir()?;
        path.push(clean);
        path.set_extension("ck");
        
        if path.is_symlink() {
            return Err(Error::new(ErrorKind::InvalidInput, "Symlinks are not supported"));
        }
        
        let content = fs::read(&path)
            .map_err(|error| Error::new(ErrorKind::Other, format!("Failed to load list file '{}': {}", path.to_string_lossy(), error)))?;
        
        Ok(Self {
            path,
            content,
        })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn iter(&self) -> ListIter {
        ListIter { content: &self.content }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn insert(&mut self, tag: &[u8], value: u64) -> io::Result<()> {
        if self.iter().any(|current| current.tag.eq_ignore_ascii_case(tag)) {
            return Err(Error::new(ErrorKind::AlreadyExists, "Tag in use"));
        }
        
        let capacity = self.content.len() + (tag.len() + MEM_SIZE * 2);
        let entries = self.iter()
            .chain(Some(ListEntry { tag, value }));
        
        self.commit(Self::serialize(capacity, entries))
    }
    
    pub fn update(&mut self, tag: &[u8], value: u64) -> io::Result<()> {
        let position = self.iter().position(|current| current.tag.eq_ignore_ascii_case(tag))
            .ok_or(Error::new(ErrorKind::NotFound, "Tag not found"))?;
        
        let capacity = self.content.len();
        let entries = self.iter()
            .enumerate()
            .filter_map(|(current, entry)| (current != position).then_some(entry))
            .chain(Some(ListEntry { tag, value }));
        
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
        
        fs::write(&tmp_path, &content)?;
        
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
        let (current, working) = self.content.split_at_checked(MEM_SIZE)?;
        let tag_size = usize::try_from(u64::from_le_bytes(unsafe { current.try_into().unwrap_unchecked() }))
            .expect("Tag size exceeded the maximum value supported by the plataform");
        
        let (current, working) = working.split_at_checked(tag_size)?;
        let tag = current;
        
        let (current, working) = working.split_at_checked(MEM_SIZE)?;
        let value = u64::from_le_bytes(unsafe { current.try_into().unwrap_unchecked() });
        
        self.content = working;
        
        Some(ListEntry {
            tag,
            value,
        })
    }
    
}
