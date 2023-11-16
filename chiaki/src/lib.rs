use std::{
    env,
    error::Error,
    fs::{ self, File },
    io::Write,
    mem,
    path::PathBuf,
    str,
};

pub struct List {
    path: PathBuf,
    content: Vec<u8>,
    modified: bool,
}

pub struct ListEntries<'c> {
    content: &'c [u8],
}

pub struct ListEntry<'c> {
    pub tag: &'c [u8],
    pub value: u64,
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
    
    
    pub fn iter(&self) -> ListEntries {
        ListEntries { content: &self.content }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn insert(&mut self, tag: &[u8], value: u64) -> Result<(), Box<dyn Error>> {
        if self.iter().any(|current| current.tag.eq_ignore_ascii_case(tag)) {
            return Err("Tag in use".into());
        }
        
        let capacity = self.content.len() + (mem::size_of::<u64>() * 2 + tag.len());
        let entries = self.iter()
            .chain(Some(ListEntry { tag, value }));
        
        self.content = Self::serialize(capacity, entries);
        self.modified = true;
        
        Ok(())
    }
    
    pub fn update(&mut self, tag: &[u8], value: u64) -> Result<(), Box<dyn Error>> {
        let position = self.iter().position(|current| current.tag.eq_ignore_ascii_case(tag))
            .ok_or("Tag not found")?;
        
        let capacity = self.content.len();
        let entries = self.iter()
            .enumerate()
            .filter_map(|(current, entry)| (current != position).then_some(entry))
            .chain(Some(ListEntry { tag, value }));
        
        self.content = Self::serialize(capacity, entries);
        self.modified = true;
        
        Ok(())
    }
    
    pub fn delete(&mut self, tag: &[u8]) -> Result<(), Box<dyn Error>> {
        let position = self.iter().position(|current| current.tag.eq_ignore_ascii_case(tag))
            .ok_or("Tag not found")?;
        
        let capacity = self.content.len() - (mem::size_of::<u64>() * 2 + tag.len());
        let entries = self.iter()
            .enumerate()
            .filter_map(|(current, entry)| (current != position).then_some(entry));
        
        self.content = Self::serialize(capacity, entries);
        self.modified = true;
        
        Ok(())
    }
    
    pub fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        let tmp_path = chikuwa::EphemeralPath::builder()
            .with_base(self.path.parent().ok_or("Invalid path")?)
            .with_suffix(".tmp")
            .build();
        
        File::create(&tmp_path)?.write_all(&self.content)?;
        
        // attempt to perform the update atomically
        fs::rename(&tmp_path, &self.path)?;
        
        tmp_path.unmanage();
        
        self.modified = false;
        
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

impl Drop for List {
    
    fn drop(&mut self) {
        if self.modified {
            self.commit().ok();
        }
    }
    
}

impl <'c>Iterator for ListEntries<'c> {
    
    type Item = ListEntry<'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mem_size = mem::size_of::<u64>();
        
        let working = self.content;
        
        let size = usize::try_from(u64::from_le_bytes(working.get(..mem_size)?.try_into().unwrap())).ok()?;
        let working = &working[mem_size..];
        
        let tag = working.get(..size)?;
        let working = &working[size..];
        
        let value = u64::from_le_bytes(working.get(..mem_size)?.try_into().unwrap());
        let working = &working[mem_size..];
        
        self.content = working;
        
        Some(ListEntry {
            tag,
            value,
        })
    }
    
}
