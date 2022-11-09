use std::{
    collections::HashMap,
    error::Error,
};

use bincode::{ Decode, Encode };

#[derive(Decode, Encode)]
pub struct Feeds {
    counter: u32,
    entries: HashMap<FeedsId, FeedsEntry>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FeedsId(u32);

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FeedsEntry {
    url: Box<str>,
}

enum UrlError {
    Empty,
    NonUnique,
}

impl Feeds {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            counter: 0,
            entries: HashMap::new(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get(&self, id: FeedsId) -> Option<&FeedsEntry> {
        self.entries.get(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&FeedsId, &FeedsEntry)> {
        self.entries.iter()
    }
    
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add(&mut self, entry: FeedsEntry) -> Result<FeedsId, Box<dyn Error>> {
        self.counter = self.counter.checked_add(1)
            .ok_or("Maximum id value reached")?;
        
        let id = FeedsId::from(self.counter);
        
        self.check_entry(id, &entry)?;
        
        self.entries.insert(id, entry);
        
        Ok(id)
    }
    
    pub fn edit(&mut self, id: FeedsId, entry: FeedsEntry) -> Result<FeedsEntry, Box<dyn Error>> {
        if ! self.entries.contains_key(&id) {
            return Err("Feed not found".into());
        }
        
        self.check_entry(id, &entry)?;
        
        Ok(self.entries.insert(id, entry).unwrap())
    }
    
    pub fn remove(&mut self, id: FeedsId) -> Result<FeedsEntry, Box<dyn Error>> {
        let entry = self.entries.remove(&id)
            .ok_or("Feed not found")?;
        
        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
            self.entries.shrink_to_fit();
        }
        
        Ok(entry)
    }
    
    
    // ---------- validators ----------    
    
    
    fn check_entry(&self, id: FeedsId, entry: &FeedsEntry) -> Result<(), Box<dyn Error>> {
        if let Err(error) = self.validate_url(id, entry) {
            match error {
                UrlError::Empty => return Err("URL: cannot be empty".into()),
                UrlError::NonUnique => return Err("URL: already defined for another entry".into()),
            }
        }
        
        Ok(())
    }
    
    fn validate_url(&self, id: FeedsId, entry: &FeedsEntry) -> Result<(), UrlError> {
        if entry.url.is_empty() {
            return Err(UrlError::Empty);
        }
        
        if self.iter().any(|(&k, v)| v.url.eq_ignore_ascii_case(&entry.url) && k != id) {
            return Err(UrlError::NonUnique);
        }
        
        Ok(())
    }
    
}

impl From<u32> for FeedsId {
    
    fn from(id: u32) -> Self {
        Self(id)
    }
    
}

impl FeedsId {
    
    pub fn as_int(self) -> u32 {
        self.0
    }
    
}

impl FeedsEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            url: Box::default(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn url(&self) -> &str {
        self.url.as_ref()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn with_url(mut self, url: String) -> Self {
        self.url = url.into_boxed_str();
        self
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod add {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            // operation
            
            let output = feeds.add(entry);
            
            // control
            
            assert!(output.is_ok());
            
            let id = output.unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            assert_eq!(feeds.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::new());
            
            // operation
            
            let output = feeds.add(entry);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod edit {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            let id = feeds.add(entry).unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://test.com/rss"));
            
            // operation
            
            let output = feeds.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://test.com/rss"));
            
            assert_eq!(feeds.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            let id = feeds.add(entry).unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::new());
            
            // operation
            
            let output = feeds.edit(id, entry);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            let id = feeds.add(entry).unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            // operation
            
            let output = feeds.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            feeds.add(entry).unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://test.com/rss"));
            
            // operation
            
            let output = feeds.edit(FeedsId::from(0), entry);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod remove {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            let id = feeds.add(entry).unwrap();
            
            // operation
            
            let output = feeds.remove(id);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(feeds.get(id).is_none());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            feeds.add(entry).unwrap();
            
            // operation
            
            let output = feeds.remove(FeedsId::from(0));
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod validators {
        
        use super::*;
        
        // url
        
        #[test]
        fn url_empty() {
            // setup
            
            let feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::new());
            
            // operation
            
            let output = feeds.check_entry(FeedsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            assert!(feeds.check_entry(FeedsId::from(0), &entry).is_ok());
        }
        
        #[test]
        fn url_non_unique() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            let id = feeds.add(entry).unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            // operation
            
            let output = feeds.check_entry(FeedsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(feeds.check_entry(id, &entry).is_ok());
        }
        
        #[test]
        fn url_non_unique_mixed_case() {
            // setup
            
            let mut feeds = Feeds::new();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://example.com/rss"));
            
            let id = feeds.add(entry).unwrap();
            
            let entry = FeedsEntry::new()
                .with_url(String::from("http://Example.com/rss"));
            
            // operation
            
            let output = feeds.check_entry(FeedsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(feeds.check_entry(id, &entry).is_ok());
        }
        
    }
    
}
