use std::error::Error;

use super::Formats;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FormatsId(i64);

impl From<i64> for FormatsId {
    
    fn from(value: i64) -> Self {
        Self(value)
    }
    
}

impl FormatsId {
    
    // ---------- accessors ----------
    
    
    pub fn as_int(self) -> i64 {
        self.0
    }
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(self, formats: &Formats, insertion: bool) -> Result<(), Box<dyn Error>> {
        if self.as_int() <= 0 {
            return Err("Id: cannot be lower than or equal to zero".into());
        }
        
        if insertion && formats.iter().any(|(&k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        Ok(())
    }
    
}
