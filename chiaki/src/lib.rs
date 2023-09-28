use std::{
    env,
    error::Error,
    fs::{ self, File },
    io::{ Write, BufWriter },
    mem,
    path::PathBuf,
    str,
};

pub struct Database {
    path: PathBuf,
    content: Vec<u8>,
}

pub struct DatabaseEntries<'c> {
    content: &'c [u8],
}

pub struct DatabaseEntry<'c> {
    pub tag: &'c str,
    pub value: u64,
}

impl Database {
    
    // -------------------- constructors --------------------
    
    
    pub fn load(name: &str) -> Result<Self, Box<dyn Error>> {
        let path = env::current_exe()?
            .with_file_name(name)
            .with_extension("ck");
        
        let content = fs::read(&path)
            .map_err(|_| chikuwa::concat_str!("Load of database file failed: '", &path.to_string_lossy(), "'"))?;
        
        Ok(Self { path, content })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn entries(&self) -> impl Iterator<Item = DatabaseEntry> {
        DatabaseEntries { content: &self.content }
    }
    
    fn commit<'c>(&self, entries: impl Iterator<Item = DatabaseEntry<'c>>) -> Result<(), Box<dyn Error>> {
        let tmp_path = chikuwa::EphemeralPath::builder()
            .with_base(self.path.parent().ok_or("Invalid path")?)
            .with_suffix(".tmp")
            .build();
        
        let mut writer = BufWriter::new(File::create(&tmp_path)?);
        
        for entry in entries {
            writer.write_all(&u64::try_from(entry.tag.len())?.to_le_bytes())?;
            writer.write_all(entry.tag.as_bytes())?;
            writer.write_all(&entry.value.to_le_bytes())?;
        }
        
        writer.flush()?;
        
        // attempt to perform the update atomically
        fs::rename(&tmp_path, &self.path)?;
        
        tmp_path.unmanage();
        
        Ok(())
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn add(self, tag: &str, value: u64) -> Result<(), Box<dyn Error>> {
        if self.entries().any(|entry| entry.tag.eq_ignore_ascii_case(tag)) {
            return Err("Tag in use".into());
        }
        
        let modified = self.entries()
            .chain(Some(DatabaseEntry { tag, value }));
        
        self.commit(modified)
    }
    
    pub fn edit(self, tag: &str, value: u64) -> Result<(), Box<dyn Error>> {
        if ! self.entries().any(|entry| entry.tag.eq_ignore_ascii_case(tag)) {
            return Err("Tag not found".into());
        }
        
        let modified = self.entries()
            .filter(|entry| ! entry.tag.eq_ignore_ascii_case(tag))
            .chain(Some(DatabaseEntry { tag, value }));
        
        self.commit(modified)
    }
    
    pub fn remove(self, tag: &str) -> Result<(), Box<dyn Error>> {
        if ! self.entries().any(|entry| entry.tag.eq_ignore_ascii_case(tag)) {
            return Err("Tag not found".into());
        }
        
        let modified = self.entries()
            .filter(|entry| ! entry.tag.eq_ignore_ascii_case(tag));
        
        self.commit(modified)
    }
    
}

impl <'c>Iterator for DatabaseEntries<'c> {
    
    type Item = DatabaseEntry<'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        let (current, rest) = self.content.get(..mem::size_of::<u64>()).zip(self.content.get(mem::size_of::<u64>()..))?;
        let size = usize::try_from(u64::from_le_bytes(current.try_into().unwrap())).ok()?;
        
        let (current, rest) = rest.get(..size).zip(rest.get(size..))?;
        let tag = str::from_utf8(current).ok()?;
        
        let (current, rest) = rest.get(..mem::size_of::<u64>()).zip(rest.get(mem::size_of::<u64>()..))?;
        let value = u64::from_le_bytes(current.try_into().unwrap());
        
        self.content = rest;
        
        Some(DatabaseEntry {
            tag,
            value,
        })
    }
    
}
