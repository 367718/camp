use std::{
    collections::HashMap,
    error::Error,
};

use bincode::{ Decode, Encode };

#[derive(Decode, Encode)]
pub struct Formats {
    counter: u32,
    entries: HashMap<FormatsId, FormatsEntry>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FormatsId(u32);

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FormatsEntry {
    pub name: String,
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
        
        // validation error
        self.check_entry(id, &entry)?;
        
        self.entries.insert(id, entry);
        
        Ok(id)
    }
    
    pub fn edit(&mut self, id: FormatsId, entry: FormatsEntry) -> Result<FormatsEntry, Box<dyn Error>> {
        if ! self.entries.contains_key(&id) {
            return Err("Format not found".into());
        }
        
        // validation error
        self.check_entry(id, &entry)?;
        
        let previous = self.entries.insert(id, entry).unwrap();
        
        Ok(previous)
    }
    
    pub fn remove(&mut self, id: FormatsId) -> Result<FormatsEntry, Box<dyn Error>> {
        let previous = self.entries.remove(&id)
            .ok_or("Format not found")?;
        
        Ok(previous)
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
        if entry.name.is_empty() {
            return Err(NameError::Empty);
        }
        
        if self.iter().any(|(&k, v)| v.name.eq_ignore_ascii_case(&entry.name) && k != id) {
            return Err(NameError::NonUnique);
        }
        
        Ok(())
    }
    
}

impl From<u32> for FormatsId {
    
    fn from(id: u32) -> Self {
        Self(id)
    }
    
}

impl FormatsId {
    
    pub fn as_int(self) -> u32 {
        self.0
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
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            // operation
            
            let output = formats.add(entry);
            
            // control
            
            assert!(output.is_ok());
            
            let id = output.unwrap();
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            assert_eq!(formats.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry {
                name: String::new(),
            };
            
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
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry {
                name: String::from("mp4"),
            };
            
            // operation
            
            let output = formats.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
            
            let entry = FormatsEntry {
                name: String::from("mp4"),
            };
            
            assert_eq!(formats.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry {
                name: String::new(),
            };
            
            // operation
            
            let output = formats.edit(id, entry);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            // operation
            
            let output = formats.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
            formats.add(entry).unwrap();
            
            let entry = FormatsEntry {
                name: String::from("avi"),
            };
            
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
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
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
            
            let entry = FormatsEntry {
                name: String::from("mkv"),
            };
            
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
            
            let mut entry = FormatsEntry {
                name: String::new(),
            };
            
            // operation
            
            let output = formats.check_entry(FormatsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            entry.name = String::from("mkv");
            
            assert!(formats.check_entry(FormatsId::from(0), &entry).is_ok());
        }
        
        #[test]
        fn name_non_unique() {
            // setup
            
            let mut formats = Formats::new();
            
            let entry = FormatsEntry {
                name: String::from("avi"),
            };
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry {
                name: String::from("avi"),
            };
            
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
            
            let entry = FormatsEntry {
                name: String::from("avi"),
            };
            
            let id = formats.add(entry).unwrap();
            
            let entry = FormatsEntry {
                name: String::from("aVi"),
            };
            
            // operation
            
            let output = formats.check_entry(FormatsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(formats.check_entry(id, &entry).is_ok());
        }
        
    }
    
}
