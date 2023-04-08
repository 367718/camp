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

impl From<FormatsId> for i64 {
    
    fn from(value: FormatsId) -> i64 {
        value.0
    }
    
}

impl FormatsId {
    
    pub(crate) fn validate(self, formats: &Formats, insertion: bool) -> Result<(), Box<dyn Error>> {
        if i64::from(self) <= 0 {
            return Err("Id: cannot be lower than or equal to 0".into());
        }
        
        if insertion && formats.iter().any(|(k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        if ! insertion && ! formats.iter().any(|(k, _)| k == self) {
            return Err("Id: does not correspond to any valid entry".into());
        }
        
        Ok(())
    }
    
}
