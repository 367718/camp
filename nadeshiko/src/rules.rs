use std::error::Error;

pub struct Rules {
    inner: Vec<RuleEntry>,
}

pub struct RuleEntry {
    tag: Vec<u8>,
    value: u64,
}

pub struct RuleUpdate<'r> {
    parent: &'r mut Rules,
    index: usize,
    episode: u64,
}

impl Rules {
    
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let content = chiaki::List::load("rules")?;
        
        let mut inner = Vec::new();
        
        for entry in &content {
            inner.push(RuleEntry {
                tag: entry.tag.to_owned(),
                value: entry.value,
            });
        }
        
        Ok(Self { inner })
    }
    
    pub fn get_update<'r>(&'r mut self, title: &[u8]) -> Option<RuleUpdate<'r>> {
        for (index, entry) in self.inner.iter().enumerate() {
            
            if ! title.starts_with(&entry.tag) {
                continue;
            }
            
            let Some(episode) = chikuwa::first_number(&title[entry.tag.len()..]) else {
                continue;
            };
            
            if episode <= entry.value {
                continue;
            }
            
            return Some(RuleUpdate {
                parent: self,
                index,
                episode,
            });
            
        }
        
        None
    }
    
}

impl RuleUpdate<'_> {
    
    pub fn execute(self) -> Result<(), Box<dyn Error>> {
        let entry = self.parent.inner
            .get_mut(self.index)
            .unwrap();
        
        chiaki::List::load("rules")
            .and_then(|list| list.set(&entry.tag, self.episode))?;
        
        entry.value = self.episode;
        
        Ok(())
    }
    
}
