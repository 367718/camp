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

impl From<KindsId> for i64 {
    
    fn from(value: KindsId) -> i64 {
        value.0
    }
    
}

impl KindsId {
    
    pub fn to_int(self) -> i64 {
        self.into()
    }
    
    pub(crate) fn validate(self, kinds: &Kinds, insertion: bool) -> Result<(), Box<dyn Error>> {
        if self.to_int() <= 0 {
            return Err("Id: cannot be lower than or equal to 0".into());
        }
        
        if insertion && kinds.iter().any(|(k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        if ! insertion && ! kinds.iter().any(|(k, _)| k == self) {
            return Err("Id: does not correspond to any valid entry".into());
        }
        
        Ok(())
    }
    
}
