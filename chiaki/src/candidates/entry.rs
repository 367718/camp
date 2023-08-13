use std::{
    collections::HashSet,
    error::Error,
};

use super::{ Candidates, CandidatesId };
use crate::{ Series, SeriesId, SeriesStatus };

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct CandidatesEntry {
    series: SeriesId,
    title: Box<str>,
    group: Box<str>,
    quality: Box<str>,
    offset: i64,
    current: CandidatesCurrent,
    downloaded: HashSet<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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

enum OffsetError {
    LowerThanZero,
}

enum DownloadedError {
    ZeroOrLower,
    CannotBeSet,
}

impl Default for CandidatesEntry {
    
    fn default() -> Self {
        Self::new()
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
            offset: i64::default(),
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
    
    pub fn offset(&self) -> i64 {
        self.offset
    }
    
    pub fn current(&self) -> CandidatesCurrent {
        self.current
    }
    
    pub fn downloaded(&self) -> &HashSet<i64> {
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
    
    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = offset;
        self
    }
    
    pub fn with_current(mut self, current: CandidatesCurrent) -> Self {
        self.current = current;
        self
    }
    
    pub fn with_downloaded(mut self, downloaded: HashSet<i64>) -> Self {
        self.downloaded = downloaded;
        self
    }
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(&self, candidates: &Candidates, series: &Series, id: CandidatesId) -> Result<(), Box<dyn Error>> {
        let mut errors = Vec::with_capacity(4);
        
        if let Err(error) = self.validate_series(candidates, series, id) {
            match error {
                SeriesError::NonUnique => errors.push("Series: already defined for another entry"),
                SeriesError::NotFound => errors.push("Series: not found"),
                SeriesError::NotWatching => errors.push("Series: status not 'Watching'"),
            }
        }
        
        if let Err(error) = self.validate_title(candidates, id) {
            match error {
                TitleError::Empty => errors.push("Title: cannot be empty"),
                TitleError::NonUnique => errors.push("Title: already defined for another entry"),
            }
        }
        
        if let Err(error) = self.validate_offset() {
            match error {
                OffsetError::LowerThanZero => errors.push("Offset: cannot be lower than 0"),
            }
        }
        
        if let Err(error) = self.validate_downloaded() {
            match error {
                DownloadedError::ZeroOrLower => errors.push("Downloaded: cannot be lower than or equal to 0"),
                DownloadedError::CannotBeSet => errors.push("Downloaded: cannot be set if not current"),
            }
        }
        
        if ! errors.is_empty() {
            return Err(errors.join("\n\n").into());
        }
        
        Ok(())
    }
    
    fn validate_series(&self, candidates: &Candidates, series: &Series, id: CandidatesId) -> Result<(), SeriesError> {
        let found = series.get(self.series())
            .ok_or(SeriesError::NotFound)?;
        
        if found.status() != SeriesStatus::Watching {
            return Err(SeriesError::NotWatching);
        }
        
        if candidates.iter().any(|(k, v)| v.series() == self.series() && k != id) {
            return Err(SeriesError::NonUnique);
        }
        
        Ok(())
    }
    
    fn validate_title(&self, candidates: &Candidates, id: CandidatesId) -> Result<(), TitleError> {
        if self.title().is_empty() {
            return Err(TitleError::Empty);
        }
        
        if candidates.iter().any(|(k, v)| v.title().eq_ignore_ascii_case(self.title()) && k != id) {
            return Err(TitleError::NonUnique);
        }
        
        Ok(())
    }
    
    fn validate_offset(&self) -> Result<(), OffsetError> {
        if self.offset() < 0 {
            return Err(OffsetError::LowerThanZero);
        }
        
        Ok(())
    }
    
    fn validate_downloaded(&self) -> Result<(), DownloadedError> {
        if self.downloaded().iter().any(|&download| download <= 0) {
            return Err(DownloadedError::ZeroOrLower);
        }
        
        if ! self.downloaded().is_empty() && self.current() == CandidatesCurrent::No {
            return Err(DownloadedError::CannotBeSet);
        }
        
        Ok(())
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

impl TryFrom<i64> for CandidatesCurrent {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: i64) -> Result<Self, Self::Error> {
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

impl From<CandidatesCurrent> for i64 {
    
    fn from(value: CandidatesCurrent) -> i64 {
        match value {
            CandidatesCurrent::No => 1,
            CandidatesCurrent::Yes => 2,
        }
    }
    
}

impl From<CandidatesCurrent> for &str {
    
    fn from(value: CandidatesCurrent) -> &'static str {
        match value {
            CandidatesCurrent::No => "No",
            CandidatesCurrent::Yes => "Yes",
        }
    }
    
}

impl CandidatesCurrent {
    
    pub fn to_int(self) -> i64 {
        self.into()
    }
    
    pub fn to_str(self) -> &'static str {
        self.into()
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
        chikuwa::insensitive_contains(
            current,
            &[&self.title, &self.group, &self.quality],
        )
    }
    
    fn clean(&self, current: &str) -> String {
        let current = current.to_ascii_lowercase();
        
        current.replacen(&self.title.to_ascii_lowercase(), "", 1)
            .replacen(&self.group.to_ascii_lowercase(), "", 1)
            .replacen(&self.quality.to_ascii_lowercase(), "", 1)
    }
    
    fn can_download(&self, episode: i64) -> bool {
        ! self.downloaded.contains(&episode)
    }
    
    fn can_update(&self, _episode: i64) -> bool {
        true
    }
    
    fn id(&self) -> i64 {
        self.series.to_int()
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
    
    fn can_download(&self, episode: i64) -> bool {
        (**self).can_download(episode)
    }
    
    fn can_update(&self, episode: i64) -> bool {
        (**self).can_update(episode)
    }
    
    fn id(&self) -> i64 {
        (**self).id()
    }
    
}
