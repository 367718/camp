mod entry;

use std::{
    collections::HashMap,
    error::Error,
};

use bincode::{ Decode, Encode };

use crate::Series;

pub use entry::{ KindsId, KindsEntry };

#[derive(Decode, Encode)]
pub struct Kinds {
    counter: u32,
    entries: HashMap<KindsId, KindsEntry>,
}

enum NameError {
    Empty,
    NonUnique,
}

impl Kinds {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            counter: 0,
            entries: HashMap::new(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get(&self, id: KindsId) -> Option<&KindsEntry> {
        self.entries.get(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&KindsId, &KindsEntry)> {
        self.entries.iter()
    }
    
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add(&mut self, entry: KindsEntry) -> Result<KindsId, Box<dyn Error>> {
        self.counter = self.counter.checked_add(1)
            .ok_or("Maximum ID value reached")?;
        
        let id = KindsId::from(self.counter);
        
        self.check_entry(id, &entry)?;
        
        self.entries.insert(id, entry);
        
        Ok(id)
    }
    
    pub fn edit(&mut self, id: KindsId, entry: KindsEntry) -> Result<KindsEntry, Box<dyn Error>> {
        if ! self.entries.contains_key(&id) {
            return Err("Kind not found".into());
        }
        
        self.check_entry(id, &entry)?;
        
        Ok(self.entries.insert(id, entry).unwrap())
    }
    
    pub fn remove(&mut self, id: KindsId, series: &Series) -> Result<KindsEntry, Box<dyn Error>> {
        if series.iter().any(|(_, curr_entry)| curr_entry.kind() == id) {
            return Err("A kind cannot be removed if a related series is defined".into());
        }
        
        let entry = self.entries.remove(&id)
            .ok_or("Kind not found")?;
        
        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
            self.entries.shrink_to_fit();
        }
        
        Ok(entry)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_entry(&self, id: KindsId, entry: &KindsEntry) -> Result<(), Box<dyn Error>> {
        if let Err(error) = self.validate_name(id, entry) {
            match error {
                NameError::Empty => return Err("Name: cannot be empty".into()),
                NameError::NonUnique => return Err("Name: already defined for another entry".into()),
            }
        }
        
        Ok(())
    }
    
    fn validate_name(&self, id: KindsId, entry: &KindsEntry) -> Result<(), NameError> {
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
    
    use crate::{
        Candidates,
        SeriesEntry, SeriesStatus, SeriesGood,
    };
    
    mod add {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            // operation
            
            let output = kinds.add(entry);
            
            // control
            
            assert!(output.is_ok());
            
            let id = output.unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            assert_eq!(kinds.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::new());
            
            // operation
            
            let output = kinds.add(entry);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod edit {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let id = kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("movie"));
            
            // operation
            
            let output = kinds.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
            
            let entry = KindsEntry::new()
                .with_name(String::from("movie"));
            
            assert_eq!(kinds.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let id = kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::new());
            
            // operation
            
            let output = kinds.edit(id, entry);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("movie"));
            
            let id = kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("movie"));
            
            // operation
            
            let output = kinds.edit(id, entry);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("special"));
            
            // operation
            
            let output = kinds.edit(KindsId::from(0), entry);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod remove {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut kinds = Kinds::new();
            let series = Series::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("special"));
            
            let id = kinds.add(entry).unwrap();
            
            // operation
            
            let output = kinds.remove(id, &series);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(kinds.get(id).is_none());
        }
        
        #[test]
        fn in_use() {
            // setup
            
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("special"));
            
            let first_id = kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let second_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(first_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            // operation
            
            let output = kinds.remove(first_id, &series);
            
            // control
            
            assert!(output.is_err());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(second_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            series.edit(series_id, entry, &kinds, &candidates).unwrap();
            
            assert!(kinds.remove(first_id, &series).is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut kinds = Kinds::new();
            let series = Series::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("special"));
            
            kinds.add(entry).unwrap();
            
            // operation
            
            let output = kinds.remove(KindsId::from(0), &series);
            
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
            
            let kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::new());
            
            // operation
            
            let output = kinds.check_entry(KindsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            assert!(kinds.check_entry(KindsId::from(0), &entry).is_ok());
        }
        
        #[test]
        fn name_non_unique() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let id = kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            // operation
            
            let output = kinds.check_entry(KindsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(kinds.check_entry(id, &entry).is_ok());
        }
        
        #[test]
        fn name_non_unique_mixed_case() {
            // setup
            
            let mut kinds = Kinds::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let id = kinds.add(entry).unwrap();
            
            let entry = KindsEntry::new()
                .with_name(String::from("Tv"));
            
            // operation
            
            let output = kinds.check_entry(KindsId::from(0), &entry);
            
            // control
            
            assert!(output.is_err());
            
            assert!(kinds.check_entry(id, &entry).is_ok());
        }
        
    }
    
}