mod entry;

use std::{
    collections::HashMap,
    error::Error,
};

use bincode::{ Decode, Encode };

pub use entry::{ FormatsId, FormatsEntry };

#[derive(Decode, Encode)]
pub struct Formats {
    counter: u32,
    entries: HashMap<FormatsId, FormatsEntry>,
}

enum NameError {
    Empty,
    NonUnique,
}

impl Formats {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            counter: 0,
            entries: HashMap::new(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get(&self, id: FormatsId) -> Option<&FormatsEntry> {
        self.entries.get(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&FormatsId, &FormatsEntry)> {
        self.entries.iter()
    }
    
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add(&mut self, entry: FormatsEntry) -> Result<FormatsId, Box<dyn Error>> {
        self.counter = self.counter.checked_add(1)
            .ok_or("Maximum id value reached")?;
        
        let id = FormatsId::from(self.counter);
        
        self.check_entry(id, &entry)?;
        
        self.entries.insert(id, entry);
        
        Ok(id)
    }
    
    pub fn edit(&mut self, id: FormatsId, entry: FormatsEntry) -> Result<FormatsEntry, Box<dyn Error>> {
        if ! self.entries.contains_key(&id) {
            return Err("Format not found".into());
        }
        
        self.check_entry(id, &entry)?;
        
        Ok(self.entries.insert(id, entry).unwrap())
    }
    
    pub fn remove(&mut self, id: FormatsId) -> Result<FormatsEntry, Box<dyn Error>> {
        let entry = self.entries.remove(&id)
            .ok_or("Format not found")?;
        
        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
            self.entries.shrink_to_fit();
        }
        
        Ok(entry)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_entry(&self, id: FormatsId, entry: &FormatsEntry) -> Result<(), Box<dyn Error>> {
        if let Err(error) = self.validate_name(id, entry) {
            match error {
                NameError::Empty => return Err("Name: cannot be empty".into()),
                NameError::NonUnique => return Err("Name: already defined for another entry".into()),
            }
        }
        
        Ok(())
    }
    
    fn validate_name(&self, id: FormatsId, entry: &FormatsEntry) -> Result<(), NameError> {
        if entry.name().is_empty() {
            return Err(NameError::Empty);
        }
        
        if self.iter().any(|(&k, v)| v.name().eq_ignore_ascii_case(entry.name()) && k != id) {
            return Err(NameError::NonUnique);
        }
        
        Ok(())
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
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            // operation
            
            let output = formats.add(entry);
            
            // control
            
            assert!(output.is_ok());
            
            let id = output.unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            assert_eq!(formats.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::new());
            
            // operation
            
            let output = formats.add(entry);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod edit {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mp4"));
            
            // operation
            
            let output = formats.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mp4"));
            
            assert_eq!(formats.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::new());
            
            // operation
            
            let output = formats.edit(id, entry);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            // operation
            
            let output = formats.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            formats.add(entry).unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("avi"));
            
            // operation
            
            let output = formats.edit(FormatsId::from(0), entry);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod remove {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            let id = formats.add(entry).unwrap();
            
            // operation
            
            let output = formats.remove(id);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(formats.get(id).is_none());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            formats.add(entry).unwrap();
            
            // operation
            
            let output = formats.remove(FormatsId::from(0));
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod validators {
        
        use super::*;
        
        // name
        
        #[test]
        fn name_empty() {
            // setup
            
            let formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::new());
            
            // operation
            
            let output = formats.check_entry(FormatsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            let entry = FormatsEntry::new()
                .with_name(String::from("mkv"));
            
            assert!(formats.check_entry(FormatsId::from(0), &entry).is_ok());
        }
        
        #[test]
        fn name_non_unique() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("avi"));
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("avi"));
            
            // operation
            
            let output = formats.check_entry(FormatsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(formats.check_entry(id, &entry).is_ok());
        }
        
        #[test]
        fn name_non_unique_mixed_case() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("avi"));
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry::new()
                .with_name(String::from("aVi"));
            
            // operation
            
            let output = formats.check_entry(FormatsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(formats.check_entry(id, &entry).is_ok());
        }
        
    }
    
}
