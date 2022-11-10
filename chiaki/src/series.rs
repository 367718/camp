use std::{
    collections::HashMap,
    error::Error,
};

use bincode::{ Decode, Encode };

use crate::{
    Kinds, KindsId,
    Candidates,
};

#[derive(Decode, Encode)]
pub struct Series {
    counter: u32,
    entries: HashMap<SeriesId, SeriesEntry>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SeriesId(u32);

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SeriesEntry {
    title: Box<str>,
    kind: KindsId,
    status: SeriesStatus,
    progress: u32,
    good: SeriesGood,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SeriesStatus {
    Watching,
    OnHold,
    PlanToWatch,
    Completed,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SeriesGood {
    Yes,
    No,
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
        if entry.title.is_empty() {
            return Err(TitleError::Empty);
        }
        
        if self.iter().any(|(&k, v)| v.title.eq_ignore_ascii_case(&entry.title) && k != id) {
            return Err(TitleError::NonUnique);
        }
        
        Ok(())
    }
    
    fn validate_kind(&self, entry: &SeriesEntry, kinds: &Kinds) -> Result<(), KindError> {
        if kinds.get(entry.kind).is_none() {
            return Err(KindError::NotFound);
        }
        
        Ok(())
    }
    
    fn validate_status(&self, id: SeriesId, entry: &SeriesEntry, candidates: &Candidates) -> Result<(), StatusError> {
        if entry.status != SeriesStatus::Watching && candidates.iter().any(|(_, v)| v.series() == id) {
            return Err(StatusError::CandidateDefined);
        }
        
        Ok(())
    }
    
    fn validate_progress(&self, entry: &SeriesEntry) -> Result<(), ProgressError> {
        match entry.status {
            
            // cannot be 0
            SeriesStatus::Watching | SeriesStatus::OnHold | SeriesStatus::Completed => {
                
                if entry.progress == 0 {
                    return Err(ProgressError::Zero);
                }
                
            },
            
            // must be 0
            SeriesStatus::PlanToWatch => {
                
                if entry.progress != 0 {
                    return Err(ProgressError::NonZero);
                }
                
            },
            
        }
        
        Ok(())
    }
    
    fn validate_good(&self, entry: &SeriesEntry) -> Result<(), GoodError> {
        if entry.good == SeriesGood::Yes && entry.status != SeriesStatus::Completed {
            return Err(GoodError::CannotBeSet);
        }
        
        Ok(())
    }
    
}

impl From<u32> for SeriesId {
    
    fn from(id: u32) -> Self {
        Self(id)
    }
    
}

impl SeriesId {
    
    pub fn as_int(self) -> u32 {
        self.0
    }
    
}

impl SeriesEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            title: Box::default(),
            kind: KindsId::from(0),
            status: SeriesStatus::Watching,
            progress: 0,
            good: SeriesGood::No,
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn title(&self) -> &str {
        &self.title
    }
    
    pub fn kind(&self) -> KindsId {
        self.kind
    }
    
    pub fn status(&self) -> SeriesStatus {
        self.status
    }
    
    pub fn progress(&self) -> u32 {
        self.progress
    }
    
    pub fn good(&self) -> SeriesGood {
        self.good
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title.into_boxed_str();
        self
    }
    
    pub fn with_kind(mut self, kind: KindsId) -> Self {
        self.kind = kind;
        self
    }
    
    pub fn with_status(mut self, status: SeriesStatus) -> Self {
        self.status = status;
        self
    }
    
    pub fn with_progress(mut self, progress: u32) -> Self {
        self.progress = progress;
        self
    }
    
    pub fn with_good(mut self, good: SeriesGood) -> Self {
        self.good = good;
        self
    }
    
}

impl TryFrom<u8> for SeriesStatus {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Watching),
            2 => Ok(Self::OnHold),
            3 => Ok(Self::PlanToWatch),
            4 => Ok(Self::Completed),
            _ => Err("Invalid value".into()),
        }
    }
    
}

impl TryFrom<&str> for SeriesStatus {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            value if value.eq_ignore_ascii_case("watching") => Ok(Self::Watching),
            value if value.eq_ignore_ascii_case("on-hold") => Ok(Self::OnHold),
            value if value.eq_ignore_ascii_case("plan to watch") => Ok(Self::PlanToWatch),
            value if value.eq_ignore_ascii_case("completed") => Ok(Self::Completed),
            _ => Err("Invalid value".into()),
        }
    }
    
}

impl SeriesStatus {
    
    pub fn as_int(&self) -> u8 {
        match self {
            Self::Watching => 1,
            Self::OnHold => 2,
            Self::PlanToWatch => 3,
            Self::Completed => 4,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::Watching => "watching",
            Self::OnHold => "on-hold",
            Self::PlanToWatch => "plan to watch",
            Self::Completed => "completed",
        }
    }
    
    pub fn display(&self) -> &str {
        match self {
            Self::Watching => "Watching",
            Self::OnHold => "On-hold",
            Self::PlanToWatch => "Plan to watch",
            Self::Completed => "Completed",
        }
    }
    
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::Watching,
            Self::OnHold,
            Self::PlanToWatch,
            Self::Completed,
        ].iter().copied()
    }
    
}

impl From<bool> for SeriesGood {
    
    fn from(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }
    
}

impl TryFrom<u8> for SeriesGood {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::No),
            2 => Ok(Self::Yes),
            _ => Err("Invalid value".into()),
        }
    }
    
}

impl TryFrom<&str> for SeriesGood {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            value if value.eq_ignore_ascii_case("no") => Ok(Self::No),
            value if value.eq_ignore_ascii_case("yes") => Ok(Self::Yes),
            _ => Err("Invalid value".into()),
        }
    }
    
}

impl SeriesGood {
    
    pub fn as_int(&self) -> u8 {
        match self {
            Self::No => 1,
            Self::Yes => 2,
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            Self::No => "no",
            Self::Yes => "yes",
        }
    }
    
    pub fn display(&self) -> &str {
        match self {
            Self::No => "No",
            Self::Yes => "Yes",
        }
    }
    
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::No,
            Self::Yes,
        ].iter().copied()
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    use crate::{
        KindsEntry,
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
