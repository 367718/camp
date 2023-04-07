use std::error::Error;

use super::Candidates;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct CandidatesId(i64);

impl From<i64> for CandidatesId {
    
    fn from(value: i64) -> Self {
        Self(value)
    }
    
}

impl CandidatesId {
    
    // ---------- accessors ----------
    
    
    pub fn as_int(self) -> i64 {
        self.0
    }
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(self, candidates: &Candidates, insertion: bool) -> Result<(), Box<dyn Error>> {
        if self.as_int() <= 0 {
            return Err("Id: cannot be lower than or equal to zero".into());
        }
        
        if insertion && candidates.iter().any(|(&k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        if ! insertion && ! candidates.iter().any(|(&k, _)| k == self) {
            return Err("Id: does not correspond to any valid entry".into());
        }
        
        Ok(())
    }
    
}
