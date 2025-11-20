use std::error::Error;

pub struct Cache<'l> {
    inner: Vec<chiaki::ListEntry<'l>>,
    max_size: u64,
}

pub struct RuleUpdate<'c, 'l> {
    cache: &'c mut Cache<'l>,
    index: usize,
    episode: u16,
}

impl<'l> Cache<'l> {
    
    pub fn new(content: &'l chiaki::List, max_size: u64) -> Self {
        Self {
            inner: content.iter().collect(),
            max_size,
        }
    }
    
    pub fn get_rule_update<'c>(&'c mut self, title: &[u8]) -> Option<RuleUpdate<'c, 'l>> {
        for (index, entry) in self.inner.iter().enumerate() {
            
            if ! title.starts_with(entry.tag) {
                continue;
            }
            
            let Some(episode) = chikuwa::first_number(&title[entry.tag.len()..]) else {
                continue;
            };
            
            if episode <= entry.value {
                continue;
            }
            
            return Some(RuleUpdate {
                cache: self,
                index,
                episode,
            });
            
        }
        
        None
    }
    
}

impl RuleUpdate<'_, '_> {
    
    pub fn execute(self) -> Result<(), Box<dyn Error>> {
        let entry = self.cache.inner
            .get_mut(self.index)
            .unwrap();
        
        chiaki::List::load("rules", self.cache.max_size)
            .and_then(|list| list.set(entry.tag, self.episode))?;
        
        entry.value = self.episode;
        
        Ok(())
    }
    
}
