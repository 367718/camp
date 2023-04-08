use std::error::Error;

use super::Feeds;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FeedsId(i64);

impl From<i64> for FeedsId {
    
    fn from(value: i64) -> Self {
        Self(value)
    }
    
}

impl From<FeedsId> for i64 {
    
    fn from(value: FeedsId) -> i64 {
        value.0
    }
    
}

impl FeedsId {
    
    pub(crate) fn validate(self, feeds: &Feeds, insertion: bool) -> Result<(), Box<dyn Error>> {
        if i64::from(self) <= 0 {
            return Err("Id: cannot be lower than or equal to 0".into());
        }
        
        if insertion && feeds.iter().any(|(k, _)| k == self) {
            return Err("Id: already in use for another entry".into());
        }
        
        if ! insertion && ! feeds.iter().any(|(k, _)| k == self) {
            return Err("Id: does not correspond to any valid entry".into());
        }
        
        Ok(())
    }
    
}
