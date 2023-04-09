mod feeds;
mod formats;
mod kinds;
mod series;
mod candidates;
mod persistence;

use std::{
    error::Error,
    path::Path,
};

use feeds::Feeds;
use formats::Formats;
use kinds::Kinds;
use series::Series;
use candidates::Candidates;
use persistence::{ Persistence, Queries, Binds, FromRow };

pub use feeds::{ FeedsId, FeedsEntry };
pub use formats::{ FormatsId, FormatsEntry };
pub use kinds::{ KindsId, KindsEntry };
pub use series::{ SeriesId, SeriesEntry, SeriesStatus, SeriesGood };
pub use candidates::{ CandidatesId, CandidatesEntry, CandidatesCurrent };

pub struct Database {
    persistence: Persistence,
    feeds: Feeds,
    formats: Formats,
    kinds: Kinds,
    series: Series,
    candidates: Candidates,
}

impl Database {
    
    // ---------- constructors ----------
    
    
    pub fn new<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn Error>> {
        let mut persistence = Persistence::new(path)?;
        let feeds = Feeds::new(&mut persistence)?;
        let formats = Formats::new(&mut persistence)?;
        let kinds = Kinds::new(&mut persistence)?;
        let series = Series::new(&mut persistence)?;
        let candidates = Candidates::new(&mut persistence)?;
        
        Ok(Self {
            persistence,
            feeds,
            formats,
            kinds,
            series,
            candidates,
        })
    }
    
    pub fn load<S: AsRef<Path>>(path: S) -> Result<Self, Box<dyn Error>> {
        let persistence = Persistence::load(path)?;
        let feeds = Feeds::load(&persistence)?;
        let formats = Formats::load(&persistence)?;
        let kinds = Kinds::load(&persistence)?;
        let series = Series::load(&persistence, &kinds)?;
        let candidates = Candidates::load(&persistence, &series)?;
        
        Ok(Self {
            persistence,
            feeds,
            formats,
            kinds,
            series,
            candidates,
        })
    }
    
}

macro_rules! api_impl {
    
    ($module: ident, $id: ident, $entry: ident $(,$related:tt: $t:ty)*) => {
        
        paste! {
            
            impl crate::Database {
                
                // ---------- accessors ----------
                
                
                pub fn [<$module _get>](&self, id: $id) -> Option<&$entry> {
                    self.$module.get(id)
                }
                
                pub fn [<$module _iter>](&self) -> impl Iterator<Item = ($id, &$entry)> {
                    self.$module.iter()
                }
                
                pub fn [<$module _count>](&self) -> usize {
                    self.$module.count()
                }
                
                
                // ---------- mutators ----------
                
                
                pub fn [<$module _add>](&mut self, entry: $entry) -> Result<$id, Box<dyn std::error::Error>> {
                    self.$module.add(&mut self.persistence $(,&self.$related)*, entry)
                }
                
                pub fn [<$module _edit>](&mut self, id: $id, entry: $entry) -> Result<$entry, Box<dyn std::error::Error>> {
                    self.$module.edit(&mut self.persistence $(,&self.$related)*, id, entry)
                }
                
                pub fn [<$module _remove>](&mut self, id: $id) -> Result<$entry, Box<dyn std::error::Error>> {
                    self.$module.remove(&mut self.persistence, id)
                }
                
                pub fn [<$module _mass_add>](&mut self, entries: impl Iterator<Item = $entry>) -> Result<(), Box<dyn std::error::Error>> {
                    self.$module.mass_add(&mut self.persistence $(,&self.$related)*, entries)
                }
                
                pub fn [<$module _mass_edit>](&mut self, entries: impl Iterator<Item = ($id, $entry)>) -> Result<(), Box<dyn std::error::Error>> {
                    self.$module.mass_edit(&mut self.persistence $(,&self.$related)*, entries)
                }
                
                pub fn [<$module _mass_remove>](&mut self, entries: impl Iterator<Item = $id>) -> Result<(), Box<dyn std::error::Error>> {
                    self.$module.mass_remove(&mut self.persistence, entries)
                }
                
                
                // ---------- validators ----------
                
                
                pub fn [<$module _validate_id>](&self, id: $id) -> Result<(), Box<dyn std::error::Error>> {
                    self.$module.validate_id(id, false)
                }
                
                pub fn [<$module _validate_entry>](&self, entry: &$entry, id: Option<$id>) -> Result<(), Box<dyn std::error::Error>> {
                    self.$module.validate_entry($(&self.$related,)* entry, id.unwrap_or_else(|| $id::from(0)))
                }
                
            }
            
        }
        
    }
    
}

pub(crate) use api_impl;

macro_rules! module_impl {
    
    ($module: ident, $id: ident, $entry: ident $(,$related:tt: $t:ty)*) => {
        
        impl $module {
            
            // ---------- constructors ----------
            
            
            pub fn new(persistence: &mut crate::Persistence) -> Result<Self, Box<dyn std::error::Error>> {
                let module = Self {
                    entries: HashMap::new(),
                };
                
                persistence.create(&module)?;
                
                Ok(module)
            }
            
            pub fn load(persistence: &crate::Persistence $(,$related: $t)*) -> Result<Self, Box<dyn std::error::Error>> {
                let mut module = Self {
                    entries: HashMap::new(),
                };
                
                module.entries.reserve(persistence.count(&module)?.try_into()?);
                
                for (id, entry) in persistence.select::<($id, $entry)>(&module)? {
                    if module.validate_id(id, true).is_ok() && module.validate_entry($($related,)* &entry, id).is_ok() {
                        module.entries.insert(id, entry);
                    }
                }
                
                Ok(module)
            }
            
            
            // ---------- accessors ----------
            
            
            pub fn get(&self, id: $id) -> Option<&$entry> {
                self.entries.get(&id)
            }
            
            pub fn iter(&self) -> impl Iterator<Item = ($id, &$entry)> {
                self.entries.iter().map(|(&id, entry)| (id, entry))
            }
            
            pub fn count(&self) -> usize {
                self.entries.len()
            }
            
            
            // ---------- mutators ----------
            
            
            pub fn add(&mut self, persistence: &mut crate::Persistence $(,$related: $t)*, entry: $entry) -> Result<$id, Box<dyn std::error::Error>> {
                let (id, entry) = self.insert_operation(persistence $(,$related)*, entry)?;
                
                self.entries.insert(id, entry);
                
                Ok(id)
            }
            
            pub fn edit(&mut self, persistence: &mut crate::Persistence $(,$related: $t)*, id: $id, entry: $entry) -> Result<$entry, Box<dyn std::error::Error>> {
                let (id, entry) = self.update_operation(persistence $(,$related)*, id, entry)?;
                
                Ok(self.entries.insert(id, entry).unwrap())
            }
            
            pub fn remove(&mut self, persistence: &mut crate::Persistence, id: $id) -> Result<$entry, Box<dyn std::error::Error>> {
                let id = self.delete_operation(persistence, id)?;
                
                let entry = self.entries.remove(&id).unwrap();
                
                if self.entries.capacity() > self.entries.len().saturating_mul(2) {
                    self.entries.shrink_to_fit();
                }
                
                Ok(entry)
            }
            
            pub fn mass_add(&mut self, persistence: &mut crate::Persistence $(,$related: $t)*, entries: impl Iterator<Item = $entry>) -> Result<(), Box<dyn std::error::Error>> {
                persistence.begin_transaction()?;
                
                let result = entries.map(|entry| self.insert_operation(persistence $(,$related)*, entry))
                    .collect::<Result<Vec<($id, $entry)>, _>>();
                
                match result {
                    
                    Ok(entries) => {
                        
                        persistence.commit();
                        
                        for (id, entry) in entries {
                            self.entries.insert(id, entry);
                        }
                        
                        Ok(())
                        
                    },
                    
                    Err(error) => {
                        
                        persistence.rollback();
                        
                        Err(error)
                        
                    },
                    
                }
            }
            
            pub fn mass_edit(&mut self, persistence: &mut crate::Persistence $(,$related: $t)*, entries: impl Iterator<Item = ($id, $entry)>) -> Result<(), Box<dyn std::error::Error>> {
                persistence.begin_transaction()?;
                
                let result = entries.map(|(id, entry)| self.update_operation(persistence $(,$related)*, id, entry))
                    .collect::<Result<Vec<($id, $entry)>, _>>();
                
                match result {
                
                    Ok(entries) => {
                        
                        persistence.commit();
                        
                        for (id, entry) in entries {
                            self.entries.insert(id, entry);
                        }
                        
                        Ok(())
                        
                    },
                    
                    Err(error) => {
                        
                        persistence.rollback();
                        
                        Err(error)
                        
                    },
                    
                }
            }
            
            pub fn mass_remove(&mut self, persistence: &mut crate::Persistence, entries: impl Iterator<Item = $id>) -> Result<(), Box<dyn std::error::Error>> {
                persistence.begin_transaction()?;
                
                let result = entries.map(|id| self.delete_operation(persistence, id))
                    .collect::<Result<Vec<$id>, _>>();
                
                match result {
            
                    Ok(entries) => {
                        
                        persistence.commit();
                        
                        for id in &entries {
                            self.entries.remove(id).unwrap();
                        }
                        
                        if self.entries.capacity() > self.entries.len().saturating_mul(2) {
                            self.entries.shrink_to_fit();
                        }
                        
                        Ok(())
                        
                    },
                    
                    Err(error) => {
                        
                        persistence.rollback();
                        
                        Err(error)
                        
                    },
                    
                }
            }
            
            fn insert_operation(&mut self, persistence: &mut crate::Persistence $(,$related: $t)*, entry: $entry) -> Result<($id, $entry), Box<dyn std::error::Error>> {
                self.validate_entry($($related,)* &entry, $id::from(0))?;
                
                let id = $id::from(persistence.insert(self, (None, Some(&entry)))?);
                
                self.validate_id(id, true)?;
                
                Ok((id, entry))
            }
            
            fn update_operation(&mut self, persistence: &mut crate::Persistence $(,$related: $t)*, id: $id, entry: $entry) -> Result<($id, $entry), Box<dyn std::error::Error>> {
                self.validate_id(id, false)?;
                self.validate_entry($($related,)* &entry, id)?;
                
                persistence.update(self, (Some(id), Some(&entry)))?;
                
                Ok((id, entry))
            }
            
            fn delete_operation(&mut self, persistence: &mut crate::Persistence, id: $id) -> Result<$id, Box<dyn std::error::Error>> {
                self.validate_id(id, false)?;
                
                persistence.delete(self, (Some(id), None))?;
                
                Ok(id)
            }
            
            
            // ---------- validators ----------
            
            
            pub fn validate_id(&self, id: $id, insertion: bool) -> Result<(), Box<dyn std::error::Error>> {
                id.validate(self, insertion)
            }
            
            pub fn validate_entry(&self $(,$related: $t)*, entry: &$entry, id: $id) -> Result<(), Box<dyn std::error::Error>> {
                entry.validate(self $(,$related)*, id)
            }
            
        }
        
    };
    
}

pub(crate) use module_impl;
