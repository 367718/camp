use bincode::{ Decode, Encode };

#[derive(Clone, Copy, Hash, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct KindsId(u32);

#[derive(Clone, PartialEq, Eq, Decode, Encode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct KindsEntry {
    name: Box<str>,
}

impl From<u32> for KindsId {
    
    fn from(id: u32) -> Self {
        Self(id)
    }
    
}

impl KindsId {
    
    pub fn as_int(self) -> u32 {
        self.0
    }
    
}

impl KindsEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            name: Box::default(),
        }
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name.into_boxed_str();
        self
    }
    
}
