use std::error::Error;

use super::{ Kinds, KindsId };

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct KindsEntry {
    name: Box<str>,
}

enum NameError {
    Empty,
    NonUnique,
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
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(&self, kinds: &Kinds, id: Option<KindsId>) -> Result<(), Box<dyn Error>> {
        if let Err(error) = self.validate_name(kinds, id) {
            match error {
                NameError::Empty => return Err("Name: cannot be empty".into()),
                NameError::NonUnique => return Err("Name: already defined for another entry".into()),
            }
        }
        
        Ok(())
    }
    
    fn validate_name(&self, kinds: &Kinds, id: Option<KindsId>) -> Result<(), NameError> {
        if self.name().is_empty() {
            return Err(NameError::Empty);
        }
        
        match id {
            
            Some(id) => if kinds.iter().any(|(&k, v)| v.name().eq_ignore_ascii_case(self.name()) && k != id) {
                return Err(NameError::NonUnique);
            },
            
            None => if kinds.iter().any(|(_, v)| v.name().eq_ignore_ascii_case(self.name())) {
                return Err(NameError::NonUnique);
            },
            
        }
        
        Ok(())
    }
    
}
