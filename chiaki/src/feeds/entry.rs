use std::error::Error;

use super::{ Feeds, FeedsId };

#[derive(Clone, PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct FeedsEntry {
    url: Box<str>,
}

enum UrlError {
    Empty,
    NonUnique,
}

impl FeedsEntry {
    
    // ---------- constructors ----------
    
    
    pub fn new() -> Self {
        Self {
            url: Box::default(),
        }
    }
    
    // ---------- accessors ----------
    
    
    pub fn url(&self) -> &str {
        &self.url
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn with_url(mut self, url: String) -> Self {
        self.url = url.into_boxed_str();
        self
    }
    
    
    // ---------- validators ----------    
    
    
    pub(crate) fn validate(&self, feeds: &Feeds, id: Option<FeedsId>) -> Result<(), Box<dyn Error>> {
        if let Err(error) = self.validate_url(feeds, id) {
            match error {
                UrlError::Empty => return Err("URL: cannot be empty".into()),
                UrlError::NonUnique => return Err("URL: already defined for another entry".into()),
            }
        }
        
        Ok(())
    }
    
    fn validate_url(&self, feeds: &Feeds, id: Option<FeedsId>) -> Result<(), UrlError> {
        if self.url().is_empty() {
            return Err(UrlError::Empty);
        }
        
        match id {
            
            Some(id) => if feeds.iter().any(|(&k, v)| v.url().eq_ignore_ascii_case(self.url()) && k != id) {
                return Err(UrlError::NonUnique);
            },
            
            None => if feeds.iter().any(|(_, v)| v.url().eq_ignore_ascii_case(self.url())) {
                return Err(UrlError::NonUnique);
            },
            
        }
        
        Ok(())
    }
    
}
