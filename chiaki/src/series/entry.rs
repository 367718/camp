use std::error::Error;

use super::{ Series, SeriesId };
use crate::{ Kinds, KindsId };

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SeriesEntry {
    title: Box<str>,
    kind: KindsId,
    status: SeriesStatus,
    progress: i64,
    good: SeriesGood,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum SeriesStatus {
    Watching,
    OnHold,
    PlanToWatch,
    Completed,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
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

enum ProgressError {
    LowerThanZero,
    Zero,
    NonZero,
}

enum GoodError {
    CannotBeSet,
}

impl Default for SeriesEntry {
    
    fn default() -> Self {
        Self::new()
    }
    
}

impl SeriesEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            title: Box::default(),
            kind: KindsId::from(0),
            status: SeriesStatus::Watching,
            progress: i64::default(),
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
    
    pub fn progress(&self) -> i64 {
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
    
    pub fn with_progress(mut self, progress: i64) -> Self {
        self.progress = progress;
        self
    }
    
    pub fn with_good(mut self, good: SeriesGood) -> Self {
        self.good = good;
        self
    }
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(&self, series: &Series, kinds: &Kinds, id: Option<SeriesId>) -> Result<(), Box<dyn Error>> {
        let mut errors = Vec::with_capacity(4);
        
        if let Err(error) = self.validate_title(series, id) {
            match error {
                TitleError::Empty => errors.push("Title: cannot be empty"),
                TitleError::NonUnique => errors.push("Title: already defined for another entry"),
            }
        }
        
        if let Err(error) = self.validate_kind(kinds) {
            match error {
                KindError::NotFound => errors.push("Kind: not found"),
            }
        }
        
        if let Err(error) = self.validate_progress() {
            match error {
                ProgressError::LowerThanZero => errors.push("Progress: cannot be lower than 0"),
                ProgressError::Zero => errors.push("Progress: must be greater than 0 for the specified status"),
                ProgressError::NonZero => errors.push("Progress: cannot be greater than 0 for the specified status"),
            }
        }
        
        if let Err(error) = self.validate_good() {
            match error {
                GoodError::CannotBeSet => errors.push("Good: cannot be set for the specified status"),
            }
        }
        
        if ! errors.is_empty() {
            return Err(errors.join("\n\n").into());
        }
        
        Ok(())
    }
    
    fn validate_title(&self, series: &Series, id: Option<SeriesId>) -> Result<(), TitleError> {
        if self.title().is_empty() {
            return Err(TitleError::Empty);
        }
        
        match id {
            
            Some(id) => if series.iter().any(|(&k, v)| v.title().eq_ignore_ascii_case(self.title()) && k != id) {
                return Err(TitleError::NonUnique);
            },
            
            None => if series.iter().any(|(_, v)| v.title().eq_ignore_ascii_case(self.title())) {
                return Err(TitleError::NonUnique);
            },
            
        }
        
        Ok(())
    }
    
    fn validate_kind(&self, kinds: &Kinds) -> Result<(), KindError> {
        if ! kinds.iter().any(|(&k, _)| k == self.kind) {
            return Err(KindError::NotFound);
        }
        
        Ok(())
    }
    
    fn validate_progress(&self) -> Result<(), ProgressError> {
        if self.progress() < 0 {
            return Err(ProgressError::LowerThanZero);
        }
        
        match self.status() {
            
            // cannot be 0
            SeriesStatus::Watching | SeriesStatus::OnHold | SeriesStatus::Completed => {
                
                if self.progress() == 0 {
                    return Err(ProgressError::Zero);
                }
                
            },
            
            // must be 0
            SeriesStatus::PlanToWatch => {
                
                if self.progress() != 0 {
                    return Err(ProgressError::NonZero);
                }
                
            },
            
        }
        
        Ok(())
    }
    
    fn validate_good(&self) -> Result<(), GoodError> {
        if self.good() == SeriesGood::Yes && self.status() != SeriesStatus::Completed {
            return Err(GoodError::CannotBeSet);
        }
        
        Ok(())
    }
    
}

impl TryFrom<i64> for SeriesStatus {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: i64) -> Result<Self, Self::Error> {
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
    
    pub fn as_int(&self) -> i64 {
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

impl TryFrom<i64> for SeriesGood {
    
    type Error = Box<dyn Error>;
    
    fn try_from(value: i64) -> Result<Self, Self::Error> {
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
    
    pub fn as_int(&self) -> i64 {
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
