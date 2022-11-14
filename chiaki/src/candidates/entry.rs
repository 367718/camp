use std::{
    collections::HashSet,
    error::Error,
};

use bincode::{ Decode, Encode };

use crate::SeriesId;

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
