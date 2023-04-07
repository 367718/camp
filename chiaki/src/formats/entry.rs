use std::error::Error;

use super::{ Formats, FormatsId };

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FormatsEntry {
    name: Box<str>,
}

enum NameError {
    Empty,
    NonUnique,
}

impl FormatsEntry {
    
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
    
    
    pub(crate) fn validate(&self, formats: &Formats, id: Option<FormatsId>) -> Result<(), Box<dyn Error>> {
        if let Err(error) = self.validate_name(formats, id) {
            match error {
                NameError::Empty => return Err("Name: cannot be empty".into()),
                NameError::NonUnique => return Err("Name: already defined for another entry".into()),
            }
        }
        
        Ok(())
    }
    
    fn validate_name(&self, formats: &Formats, id: Option<FormatsId>) -> Result<(), NameError> {
        if self.name().is_empty() {
            return Err(NameError::Empty);
        }
        
        match id {
            
            Some(id) => if formats.iter().any(|(&k, v)| v.name().eq_ignore_ascii_case(self.name()) && k != id) {
                return Err(NameError::NonUnique);
            },
            
            None => if formats.iter().any(|(_, v)| v.name().eq_ignore_ascii_case(self.name())) {
                return Err(NameError::NonUnique);
            },
            
        }
        
        Ok(())
    }
    
}
