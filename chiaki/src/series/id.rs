use std::error::Error;

use super::Series;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct SeriesId(i64);

impl From<i64> for SeriesId {
    
    fn from(value: i64) -> Self {
        Self(value)
    }
    
}

impl From<SeriesId> for i64 {
    
    fn from(value: SeriesId) -> i64 {
        value.0
    }
    
}

impl SeriesId {
    
    pub fn to_int(self) -> i64 {
        self.into()
    }
    
    pub(crate) fn validate(self, series: &Series, insertion: bool) -> Result<(), Box<dyn Error>> {
        if self.to_int() <= 0 {
            return Err("Id: cannot be lower than or equal to 0".into());
        }
        
        if insertion && series.iter().any(|(k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        if ! insertion && ! series.iter().any(|(k, _)| k == self) {
            return Err("Id: does not correspond to any valid entry".into());
        }
        
        Ok(())
    }
    
}
