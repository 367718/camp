mod entry;

use std::{
    collections::HashMap,
    error::Error,
};

use bincode::{ Decode, Encode };

use crate::{ Kinds, Candidates };

pub use entry::{ SeriesId, SeriesEntry, SeriesStatus, SeriesGood };

#[derive(Decode, Encode)]
pub struct Series {
    counter: u32,
    entries: HashMap<SeriesId, SeriesEntry>,
}

enum TitleError {
    Empty,
    NonUnique,
}

enum KindError {
    NotFound,
}

enum StatusError {
    CandidateDefined,
}

enum ProgressError {
    Zero,
    NonZero,
}

enum GoodError {
    CannotBeSet,
}

impl Series {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            counter: 0,
            entries: HashMap::new(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get(&self, id: SeriesId) -> Option<&SeriesEntry> {
        self.entries.get(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&SeriesId, &SeriesEntry)> {
        self.entries.iter()
    }
    
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add(&mut self, entry: SeriesEntry, kinds: &Kinds, candidates: &Candidates) -> Result<SeriesId, Box<dyn Error>> {
        self.counter = self.counter.checked_add(1)
            .ok_or("Maximum id value reached")?;
        
        let id = SeriesId::from(self.counter);
        
        self.check_entry(id, &entry, kinds, candidates)?;
        
        self.entries.insert(id, entry);
        
        Ok(id)
    }
    
    pub fn edit(&mut self, id: SeriesId, entry: SeriesEntry, kinds: &Kinds, candidates: &Candidates) -> Result<SeriesEntry, Box<dyn Error>> {
        if ! self.entries.contains_key(&id) {
            return Err("Series not found".into());
        }
        
        self.check_entry(id, &entry, kinds, candidates)?;
        
        Ok(self.entries.insert(id, entry).unwrap())
    }
    
    pub fn remove(&mut self, id: SeriesId, candidates: &Candidates) -> Result<SeriesEntry, Box<dyn Error>> {
        if candidates.iter().any(|(_, curr_entry)| curr_entry.series() == id) {
            return Err("A series cannot be removed if a related candidate is defined".into());
        }
        
        let entry = self.entries.remove(&id)
            .ok_or("Series not found")?;
        
        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
            self.entries.shrink_to_fit();
        }
        
        Ok(entry)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_entry(&self, id: SeriesId, entry: &SeriesEntry, kinds: &Kinds, candidates: &Candidates) -> Result<(), Box<dyn Error>> {
        let mut errors = Vec::with_capacity(5);
        
        if let Err(error) = self.validate_title(id, entry) {
            match error {
                TitleError::Empty => errors.push("Title: cannot be empty"),
                TitleError::NonUnique => errors.push("Title: already defined for another entry"),
            }
        }
        
        if let Err(error) = self.validate_kind(entry, kinds) {
            match error {
                KindError::NotFound => errors.push("Kind: not found"),
            }
        }
        
        if let Err(error) = self.validate_status(id, entry, candidates) {
            match error {
                StatusError::CandidateDefined => errors.push("Status: cannot be changed if a related Candidate is defined"),
            }
        }
        
        if let Err(error) = self.validate_progress(entry) {
            match error {
                ProgressError::Zero => errors.push("Progress: must be greater than 0 for the specified status"),
                ProgressError::NonZero => errors.push("Progress: cannot be greater than 0 for the specified status"),
            }
        }
        
        if let Err(error) = self.validate_good(entry) {
            match error {
                GoodError::CannotBeSet => errors.push("Good: cannot be set for the specified status"),
            }
        }
        
        if ! errors.is_empty() {
            return Err(errors.join("\n\n").into());
        }
        
        Ok(())
    }
    
    fn validate_title(&self, id: SeriesId, entry: &SeriesEntry) -> Result<(), TitleError> {
        if entry.title().is_empty() {
            return Err(TitleError::Empty);
        }
        
        if self.iter().any(|(&k, v)| v.title().eq_ignore_ascii_case(entry.title()) && k != id) {
            return Err(TitleError::NonUnique);
        }
        
        Ok(())
    }
    
    fn validate_kind(&self, entry: &SeriesEntry, kinds: &Kinds) -> Result<(), KindError> {
        if kinds.get(entry.kind()).is_none() {
            return Err(KindError::NotFound);
        }
        
        Ok(())
    }
    
    fn validate_status(&self, id: SeriesId, entry: &SeriesEntry, candidates: &Candidates) -> Result<(), StatusError> {
        if entry.status() != SeriesStatus::Watching && candidates.iter().any(|(_, v)| v.series() == id) {
            return Err(StatusError::CandidateDefined);
        }
        
        Ok(())
    }
    
    fn validate_progress(&self, entry: &SeriesEntry) -> Result<(), ProgressError> {
        match entry.status() {
            
            // cannot be 0
            SeriesStatus::Watching | SeriesStatus::OnHold | SeriesStatus::Completed => {
                
                if entry.progress() == 0 {
                    return Err(ProgressError::Zero);
                }
                
            },
            
            // must be 0
            SeriesStatus::PlanToWatch => {
                
                if entry.progress() != 0 {
                    return Err(ProgressError::NonZero);
                }
                
            },
            
        }
        
        Ok(())
    }
    
    fn validate_good(&self, entry: &SeriesEntry) -> Result<(), GoodError> {
        if entry.good() == SeriesGood::Yes && entry.status() != SeriesStatus::Completed {
            return Err(GoodError::CannotBeSet);
        }
        
        Ok(())
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    use crate::{
        KindsId, KindsEntry,
        CandidatesEntry, CandidatesCurrent,
    };
    
    use std::collections::HashSet;
    
    mod add {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.add(entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_ok());
            
            let id = output.unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            assert_eq!(series.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::Yes);
            
            // operation
            
            let output = series.add(entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod edit {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Another series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.edit(id, entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_ok());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Another series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            assert_eq!(series.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::new())
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.edit(id, entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.edit(id, entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Another series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.edit(SeriesId::from(0), entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod remove {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let id = series.add(entry, &kinds, &candidates).unwrap();
            
            // operation
            
            let output = series.remove(id, &candidates);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(series.get(id).is_none());
        }
        
        #[test]
        fn in_use() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let mut candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            let candidate_id = candidates.add(entry, &series).unwrap();
            
            // operation
            
            let output = series.remove(series_id, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            candidates.remove(candidate_id).unwrap();
            
            assert!(series.remove(series_id, &candidates).is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            series.add(entry, &kinds, &candidates).unwrap();
            
            // operation
            
            let output = series.remove(SeriesId::from(0), &candidates);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod validators {
        
        use super::*;
        
        // title
        
        #[test]
        fn title_empty() {
            // setup
            
            let series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::new())
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            assert!(series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates).is_ok());
        }
        
        #[test]
        fn title_non_unique() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(6)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            assert!(series.check_entry(series_id, &entry, &kinds, &candidates).is_ok());
        }
        
        #[test]
        fn title_non_unique_mixed_case() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("CurrenT series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(6)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            assert!(series.check_entry(series_id, &entry, &kinds, &candidates).is_ok());
        }
        
        // kind
        
        #[test]
        fn kind_not_found() {
            // setup
            
            let series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(KindsId::from(kind_id.as_int() + 1))
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            assert!(series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates).is_ok());
        }
        
        // status
        
        #[test]
        fn candidate_defined() {
            // setup
            
            let mut series = Series::new();
            let mut kinds = Kinds::new();
            let mut candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Test"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            let candidate_id = candidates.add(entry, &series).unwrap();
            
            let entry = series.get(series_id).unwrap().clone()
                .with_status(SeriesStatus::OnHold);
            
            // operation
            
            let output = series.check_entry(series_id, &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            candidates.remove(candidate_id).unwrap();
            
            assert!(series.check_entry(series_id, &entry, &kinds, &candidates).is_ok());
        }
        
        // progress
        
        #[test]
        fn progress_zero() {
            // setup
            
            let series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(0)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::PlanToWatch)
                .with_progress(0)
                .with_good(SeriesGood::No);
            
            assert!(series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates).is_ok());
        }
        
        #[test]
        fn progress_non_zero() {
            // setup
            
            let series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::PlanToWatch)
                .with_progress(10)
                .with_good(SeriesGood::No);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(10)
                .with_good(SeriesGood::No);
            
            assert!(series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates).is_ok());
        }
        
        // good
        
        #[test]
        fn good_cannot_be_set() {
            // setup
            
            let series = Series::new();
            let mut kinds = Kinds::new();
            let candidates = Candidates::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(10)
                .with_good(SeriesGood::Yes);
            
            // operation
            
            let output = series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates);
            
            // control
            
            assert!(output.is_err());
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            assert!(series.check_entry(SeriesId::from(0), &entry, &kinds, &candidates).is_ok());
        }
        
    }
    
}
