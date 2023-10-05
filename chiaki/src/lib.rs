use std::{
    env,
    error::Error,
    fs::{ self, File },
    io::Write,
    mem,
    path::{ Path, PathBuf },
    str,
};

pub struct List {
    path: PathBuf,
    content: Vec<u8>,
    modified: bool,
}

pub struct ListEntry<'c> {
    pub tag: &'c str,
    pub value: u64,
}

pub struct ContentEntries<'c> {
    content: &'c [u8],
}

impl List {
    
    // -------------------- constructors --------------------
    
    
    pub fn load(name: &str) -> Result<Self, Box<dyn Error>> {
        let path = env::current_exe()?
            .with_file_name(name)
            .with_extension("ck");
        
        let content = fs::read(&path)
            .map_err(|error| chikuwa::concat_str!("Load of list file located at '", &path.to_string_lossy(), "' failed: '", &error.to_string(), "'"))?;
        
        Ok(Self {
            path,
            content,
            modified: false,
        })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn iter(&self) -> impl Iterator<Item = ListEntry> {
        self.entries().filter_map(ListEntry::from_content)
    }
    
    fn entries(&self) -> ContentEntries {
        ContentEntries { content: &self.content }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn insert(&mut self, tag: &str, value: u64) -> Result<(), Box<dyn Error>> {
        let bytes = tag.as_bytes();
        
        if self.entries().any(|(tag, _)| tag.eq_ignore_ascii_case(bytes)) {
            return Err("Tag in use".into());
        }
        
        let modified = self.iter()
            .chain(Some(ListEntry { tag, value }));
        
        let adjustment = mem::size_of::<u64>() * 2 + tag.len();
        
        self.content = Self::serialize(self.content.len() + adjustment, modified);
        self.modified = true;
        
        Ok(())
    }
    
    pub fn update(&mut self, tag: &str, value: u64) -> Result<(), Box<dyn Error>> {
        let bytes = tag.as_bytes();
        
        if ! self.entries().any(|(tag, _)| tag.eq_ignore_ascii_case(bytes)) {
            return Err("Tag not found".into());
        }
        
        let modified = self.iter()
            .filter(|entry| ! entry.tag.eq_ignore_ascii_case(tag))
            .chain(Some(ListEntry { tag, value }));
        
        self.content = Self::serialize(self.content.len(), modified);
        self.modified = true;
        
        Ok(())
    }
    
    pub fn delete(&mut self, tag: &str) -> Result<(), Box<dyn Error>> {
        let bytes = tag.as_bytes();
        
        if ! self.entries().any(|(tag, _)| tag.eq_ignore_ascii_case(bytes)) {
            return Err("Tag not found".into());
        }
        
        let modified = self.iter()
            .filter(|entry| ! entry.tag.eq_ignore_ascii_case(tag));
        
        let adjustment = mem::size_of::<u64>() * 2 + tag.len();
        
        self.content = Self::serialize(self.content.len() - adjustment, modified);
        self.modified = true;
        
        Ok(())
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn serialize<'c>(capacity: usize, entries: impl Iterator<Item = ListEntry<'c>>) -> Vec<u8> {
        let mut content = Vec::with_capacity(capacity);
        
        for entry in entries {
            content.extend_from_slice(&u64::try_from(entry.tag.len()).unwrap().to_le_bytes());
            content.extend_from_slice(entry.tag.as_bytes());
            content.extend_from_slice(&entry.value.to_le_bytes());
        }
        
        content
    }
    
}

impl Drop for List {
    
    fn drop(&mut self) {
        
        fn commit(path: &Path, content: &[u8]) -> Result<(), Box<dyn Error>> {
            let tmp_path = chikuwa::EphemeralPath::builder()
                .with_base(path.parent().ok_or("Invalid path")?)
                .with_suffix(".tmp")
                .build();
            
            File::create(&tmp_path)?.write_all(content)?;
            
            // attempt to perform the update atomically
            fs::rename(&tmp_path, path)?;
            
            tmp_path.unmanage();
            
            Ok(())
        }
        
        if self.modified {
            commit(&self.path, &self.content).ok();
        }
        
    }
    
}

impl<'c> ListEntry<'c> {
    
    fn from_content(content: (&'c [u8], &'c [u8])) -> Option<Self> {
        str::from_utf8(content.0)
            .ok()
            .map(|tag| Self {
                tag,
                value: u64::from_le_bytes(content.1.try_into().unwrap()),
            })
    }
    
}

impl <'c>Iterator for ContentEntries<'c> {
    
    type Item = (&'c [u8], &'c [u8]);
    
    fn next(&mut self) -> Option<Self::Item> {
        let mem_size = mem::size_of::<u64>();
        
        let (current, rest) = self.content.get(..mem_size).zip(self.content.get(mem_size..))?;
        let size = usize::try_from(u64::from_le_bytes(current.try_into().unwrap())).ok()?;
        
        let (tag, rest) = rest.get(..size).zip(rest.get(size..))?;
        let (value, rest) = rest.get(..mem_size).zip(rest.get(mem_size..))?;
        
        self.content = rest;
        
        Some((tag, value))
    }
    
}
