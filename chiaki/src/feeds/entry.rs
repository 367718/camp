use bincode::{ Decode, Encode };

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FeedsId(u32);

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FeedsEntry {
    url: Box<str>,
}

impl From<u32> for FeedsId {
    
    fn from(id: u32) -> Self {
        Self(id)
    }
    
}

impl FeedsId {
    
    pub fn as_int(self) -> u32 {
        self.0
    }
    
}

impl FeedsEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            url: Box::default(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn with_url(mut self, url: String) -> Self {
        self.url = url.into_boxed_str();
        self
    }
    
}
