use std::error::Error;

use bincode::{ Decode, Encode };

use crate::KindsId;

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
