use std::{
    collections::{ HashMap, HashSet },
    error::Error,
};

use bincode::{ Decode, Encode };

use crate::{ Series, SeriesId, SeriesStatus };

#[derive(Decode, Encode)]
pub struct Candidates {
    counter: u32,
    entries: HashMap<CandidatesId, CandidatesEntry>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct CandidatesId(u32);

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct CandidatesEntry {
    series: SeriesId,
    title: Box<str>,
    group: Box<str>,
    quality: Box<str>,
    offset: u32,
    current: CandidatesCurrent,
    downloaded: HashSet<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum CandidatesCurrent {
    Yes,
    No,
}

enum SeriesError {
    NonUnique,
    NotFound,
    NotWatching,
}

enum TitleError {
    Empty,
    NonUnique,
}

enum DownloadedError {
    Zero,
    CannotBeSet,
}

impl Candidates {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            counter: 0,
            entries: HashMap::new(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn get(&self, id: CandidatesId) -> Option<&CandidatesEntry> {
        self.entries.get(&id)
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&CandidatesId, &CandidatesEntry)> {
        self.entries.iter()
    }
    
    pub fn count(&self) -> usize {
        self.entries.len()
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn add(&mut self, entry: CandidatesEntry, series: &Series) -> Result<CandidatesId, Box<dyn Error>> {
        self.counter = self.counter.checked_add(1)
            .ok_or("Maximum id value reached")?;
        
        let id = CandidatesId::from(self.counter);
        
        self.check_entry(id, &entry, series)?;
        
        self.entries.insert(id, entry);
        
        Ok(id)
    }
    
    pub fn edit(&mut self, id: CandidatesId, entry: CandidatesEntry, series: &Series) -> Result<CandidatesEntry, Box<dyn Error>> {
        if ! self.entries.contains_key(&id) {
            return Err("Candidate not found".into());
        }
        
        self.check_entry(id, &entry, series)?;
        
        Ok(self.entries.insert(id, entry).unwrap())
    }
    
    pub fn remove(&mut self, id: CandidatesId) -> Result<CandidatesEntry, Box<dyn Error>> {
        let entry = self.entries.remove(&id)
            .ok_or("Candidate not found")?;
        
        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
            self.entries.shrink_to_fit();
        }
        
        Ok(entry)
    }
    
    
    // ---------- validators ----------
    
    
    fn check_entry(&self, id: CandidatesId, entry: &CandidatesEntry, series: &Series) -> Result<(), Box<dyn Error>> {
        let mut errors = Vec::with_capacity(3);
        
        if let Err(error) = self.validate_series(id, entry, series) {
            match error {
                SeriesError::NonUnique => errors.push("Series: already defined for another entry"),
                SeriesError::NotFound => errors.push("Series: not found"),
                SeriesError::NotWatching => errors.push("Series: status not 'Watching'"),
            }
        }
        
        if let Err(error) = self.validate_title(id, entry) {
            match error {
                TitleError::Empty => errors.push("Title: cannot be empty"),
                TitleError::NonUnique => errors.push("Title: already defined for another entry"),
            }
        }
        
        if let Err(error) = self.validate_downloaded(entry) {
            match error {
                DownloadedError::Zero => errors.push("Downloaded: cannot be 0"),
                DownloadedError::CannotBeSet => errors.push("Downloaded: cannot be set if not current"),
            }
        }
        
        if ! errors.is_empty() {
            return Err(errors.join("\n\n").into());
        }
        
        Ok(())
    }
    
    fn validate_series(&self, id: CandidatesId, entry: &CandidatesEntry, series: &Series) -> Result<(), SeriesError> {
        let found = series.get(entry.series)
            .ok_or(SeriesError::NotFound)?;
        
        if found.status() != SeriesStatus::Watching {
            return Err(SeriesError::NotWatching);
        }
        
        if self.iter().any(|(&k, v)| v.series == entry.series && k != id) {
            return Err(SeriesError::NonUnique);
        }
        
        Ok(())
    }
    
    fn validate_title(&self, id: CandidatesId, entry: &CandidatesEntry) -> Result<(), TitleError> {
        if entry.title.is_empty() {
            return Err(TitleError::Empty);
        }
        
        if self.iter().any(|(&k, v)| v.title.eq_ignore_ascii_case(&entry.title) && k != id) {
            return Err(TitleError::NonUnique);
        }
        
        Ok(())
    }
    
    fn validate_downloaded(&self, entry: &CandidatesEntry) -> Result<(), DownloadedError> {
        if entry.downloaded.contains(&0) {
            return Err(DownloadedError::Zero);
        }
        
        if ! entry.downloaded.is_empty() && entry.current == CandidatesCurrent::No {
            return Err(DownloadedError::CannotBeSet);
        }
        
        Ok(())
    }
    
}

impl From<u32> for CandidatesId {
    
    fn from(id: u32) -> Self {
        Self(id)
    }
    
}

impl CandidatesId {
    
    pub fn as_int(self) -> u32 {
        self.0
    }
    
}

impl CandidatesEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            series: SeriesId::from(0),
            title: Box::default(),
            group: Box::default(),
            quality: Box::default(),
            offset: u32::default(),
            current: CandidatesCurrent::Yes,
            downloaded: HashSet::default(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn series(&self) -> SeriesId {
        self.series
    }
    
    pub fn title(&self) -> &str {
        &self.title
    }
    
    pub fn group(&self) -> &str {
        &self.group
    }
    
    pub fn quality(&self) -> &str {
        &self.quality
    }
    
    pub fn offset(&self) -> u32 {
        self.offset
    }
    
    pub fn current(&self) -> CandidatesCurrent {
        self.current
    }
    
    pub fn downloaded(&self) -> &HashSet<u32> {
        &self.downloaded
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn with_series(mut self, series: SeriesId) -> Self {
        self.series = series;
        self
    }
    
    pub fn with_title(mut self, title: String) -> Self {
        self.title = title.into_boxed_str();
        self
    }
    
    pub fn with_group(mut self, group: String) -> Self {
        self.group = group.into_boxed_str();
        self
    }
    
    pub fn with_quality(mut self, quality: String) -> Self {
        self.quality = quality.into_boxed_str();
        self
    }
    
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_current(mut self, current: CandidatesCurrent) -> Self {
        self.current = current;
        self
    }
    
    pub fn with_downloaded(mut self, downloaded: HashSet<u32>) -> Self {
        self.downloaded = downloaded;
        self
    }
    
}

impl From<bool> for CandidatesCurrent {
    
    fn from(value: bool) -> Self {
        if value {
            Self::Yes
        } else {
            Self::No
        }
    }
    
}

impl TryFrom<u8> for CandidatesCurrent {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::No),
            2 => Ok(Self::Yes),
            _ => Err("Invalid value".into()),
        }
    }
    
}

impl TryFrom<&str> for CandidatesCurrent {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            value if value.eq_ignore_ascii_case("no") => Ok(Self::No),
            value if value.eq_ignore_ascii_case("yes") => Ok(Self::Yes),
            _ => Err("Invalid value".into()),
        }
    }
    
}

impl CandidatesCurrent {
    
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

#[cfg(feature = "nadeshiko")]
impl nadeshiko::IsCandidate for CandidatesEntry {
    
    fn is_relevant(&self, current: &str) -> bool {
        let current = current.to_ascii_lowercase();
        
        current.contains(&self.title.to_ascii_lowercase()) &&
            current.contains(&self.group.to_ascii_lowercase()) &&
            current.contains(&self.quality.to_ascii_lowercase())
    }
    
    fn clean(&self, current: &str) -> String {
        let current = current.to_ascii_lowercase();
        
        current.replacen(&self.title.to_ascii_lowercase(), "", 1)
            .replacen(&self.group.to_ascii_lowercase(), "", 1)
            .replacen(&self.quality.to_ascii_lowercase(), "", 1)
    }
    
    fn can_download(&self, episode: u32) -> bool {
        ! self.downloaded.contains(&episode)
    }
    
    fn can_update(&self, _episode: u32) -> bool {
        true
    }
    
    fn id(&self) -> u32 {
        self.series.as_int()
    }
    
}

#[cfg(feature = "nadeshiko")]
impl nadeshiko::IsCandidate for &'_ CandidatesEntry {
    
    fn is_relevant(&self, current: &str) -> bool {
        (**self).is_relevant(current)
    }
    
    fn clean(&self, current: &str) -> String {
        (**self).clean(current)
    }
    
    fn can_download(&self, episode: u32) -> bool {
        (**self).can_download(episode)
    }
    
    fn can_update(&self, episode: u32) -> bool {
        (**self).can_update(episode)
    }
    
    fn id(&self) -> u32 {
        (**self).id()
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    use crate::{
        Kinds, KindsEntry,
        SeriesEntry, SeriesGood,
    };
    
    mod add {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            // operation
            
            let output = candidates.add(entry, &series);
            
            // control
            
            assert!(output.is_ok());
            
            let id = output.unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            assert_eq!(candidates.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
                .with_title(String::new())
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.add(entry, &series);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod edit {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            let id = candidates.add(entry, &series).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Nothing"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.edit(id, entry, &series);
            
            // control
            
            assert!(output.is_ok());
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Nothing"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            assert_eq!(candidates.get(id), Some(&entry));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            let id = candidates.add(entry, &series).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::new())
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.edit(id, entry, &series);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn no_change() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            let id = candidates.add(entry, &series).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.edit(id, entry, &series);
            
            // control
            
            assert!(output.is_ok());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            candidates.add(entry, &series).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Nothing"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.edit(CandidatesId::from(0), entry, &series);
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod remove {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            let id = candidates.add(entry, &series).unwrap();
            
            // operation
            
            let output = candidates.remove(id);
            
            // control
            
            assert!(output.is_ok());
            
            assert!(candidates.get(id).is_none());
        }
        
        #[test]
        fn not_found() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            candidates.add(entry, &series).unwrap();
            
            // operation
            
            let output = candidates.remove(CandidatesId::from(0));
            
            // control
            
            assert!(output.is_err());
        }
        
    }
    
    mod validators {
        
        use super::*;
        
        // series
        
        #[test]
        fn series_non_unique() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Another"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            assert!(candidates.check_entry(candidate_id, &entry, &series).is_ok());
        }
        
        #[test]
        fn series_not_found() {
            // setup
            
            let candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
                .with_series(SeriesId::from(50))
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            assert!(candidates.check_entry(CandidatesId::from(0), &entry, &series).is_ok());
        }
        
        #[test]
        fn series_not_watching() {
            // setup
            
            let candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Completed)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Another"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            let series_entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            series.edit(series_id, series_entry, &kinds, &candidates).unwrap();
            
            assert!(candidates.check_entry(CandidatesId::from(0), &entry, &series).is_ok());
        }
        
        // title
        
        #[test]
        fn title_empty() {
            // setup
            
            let candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
                .with_title(String::new())
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Something"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            assert!(candidates.check_entry(CandidatesId::from(0), &entry, &series).is_ok());
        }
        
        #[test]
        fn title_non_unique() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let first_series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Another series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let second_series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(first_series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            let candidate_id = candidates.add(entry, &series).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(second_series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Some other Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(10)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            assert!(candidates.check_entry(candidate_id, &entry, &series).is_ok());
        }
        
        #[test]
        fn title_non_unique_mixed_case() {
            // setup
            
            let mut candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
            let entry = KindsEntry::new()
                .with_name(String::from("tv"));
            
            let kind_id = kinds.add(entry).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Current series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let first_series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = SeriesEntry::new()
                .with_title(String::from("Another series"))
                .with_kind(kind_id)
                .with_status(SeriesStatus::Watching)
                .with_progress(5)
                .with_good(SeriesGood::No);
            
            let second_series_id = series.add(entry, &kinds, &candidates).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(first_series_id)
                .with_title(String::from("Placeholder"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            let candidate_id = candidates.add(entry, &series).unwrap();
            
            let entry = CandidatesEntry::new()
                .with_series(second_series_id)
                .with_title(String::from("PlaceholdeR"))
                .with_group(String::from("Some other group"))
                .with_quality(String::from("144p"))
                .with_offset(10)
                .with_current(CandidatesCurrent::No)
                .with_downloaded(HashSet::new());
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            assert!(candidates.check_entry(candidate_id, &entry, &series).is_ok());
        }
        
        // downloaded
        
        #[test]
        fn downloaded_zero() {
            // setup
            
            let candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
                .with_current(CandidatesCurrent::Yes)
                .with_downloaded(HashSet::from([10, 11, 0, 12]));
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Test"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::Yes)
                .with_downloaded(HashSet::from([10, 11, 12]));
            
            assert!(candidates.check_entry(CandidatesId::from(0), &entry, &series).is_ok());
        }
        
        #[test]
        fn downloaded_cannot_be_set() {
            // setup
            
            let candidates = Candidates::new();
            let mut kinds = Kinds::new();
            let mut series = Series::new();
            
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
                .with_downloaded(HashSet::from([10, 11, 12]));
            
            // operation
            
            let output = candidates.check_entry(CandidatesId::from(0), &entry, &series);
            
            // control
            
            assert!(output.is_err());
            
            let entry = CandidatesEntry::new()
                .with_series(series_id)
                .with_title(String::from("Test"))
                .with_group(String::from("Nobody"))
                .with_quality(String::from("144p"))
                .with_offset(0)
                .with_current(CandidatesCurrent::Yes)
                .with_downloaded(HashSet::from([10, 11, 12]));
            
            assert!(candidates.check_entry(CandidatesId::from(0), &entry, &series).is_ok());
        }
        
    }
    
}
