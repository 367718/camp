use std::error::Error;

use super::Kinds;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct KindsId(i64);

impl From<i64> for KindsId {
    
    fn from(value: i64) -> Self {
        Self(value)
    }
    
}

impl KindsId {
    
    // ---------- accessors ----------
    
    
    pub fn as_int(self) -> i64 {
        self.0
    }
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(self, kinds: &Kinds, insertion: bool) -> Result<(), Box<dyn Error>> {
        if self.as_int() <= 0 {
            return Err("Id: cannot be lower than or equal to zero".into());
        }
        
        if insertion && kinds.iter().any(|(&k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        Ok(())
    }
    
}
