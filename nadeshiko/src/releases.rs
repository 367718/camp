use std::{
    error::Error,
    io::Read,
};

pub struct Releases {
    content: Vec<u8>,
}

pub struct ReleasesIter<'c> {
    rest: &'c [u8],
}

pub struct ReleasesEntry<'c> {
    pub title: &'c [u8],
    pub link: &'c [u8],
}

const ITEM_OPEN_TAG: &[u8] = b"<item>";
const ITEM_CLOSE_TAG: &[u8] = b"</item>";
const TITLE_OPEN_TAG: &[u8] = b"<title>";
const TITLE_CLOSE_TAG: &[u8] = b"</title>";
const LINK_OPEN_TAG: &[u8] = b"<link>";
const LINK_CLOSE_TAG: &[u8] = b"</link>";

impl Releases {
    
    pub fn new(client: &mut akari::Client, url: &str) -> Result<Self, Box<dyn Error>> {
        let mut payload = client.get(url)?;
        
        let mut content = Vec::with_capacity(payload.content_length());
        payload.read_to_end(&mut content)?;
        
        Ok(Self {
            content,
        })
    }
    
    pub fn iter(&self) -> ReleasesIter {
        ReleasesIter {
            rest: &self.content,
        }
    }
    
}

impl<'c> Iterator for ReleasesIter<'c> {
    
    type Item = ReleasesEntry<'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // expected structure
        
        // ...
        // <item>
        // ...
        // <title>...</title>
        // ...
        // <link>...</link>
        // ...
        // </item>
        // ...
        
        while let Some(range) = chikuwa::subslice_range(self.rest, ITEM_OPEN_TAG, ITEM_CLOSE_TAG) {
            
            let item = &self.rest[range.start..range.end];
            self.rest = &self.rest[range.end..][ITEM_CLOSE_TAG.len()..];
            
            let Some(title) = chikuwa::subslice_range(item, TITLE_OPEN_TAG, TITLE_CLOSE_TAG) else {
                continue;
            };
            
            let Some(link) = chikuwa::subslice_range(item, LINK_OPEN_TAG, LINK_CLOSE_TAG) else {
                continue;
            };
            
            return Some(Self::Item {
                title: &item[title],
                link: &item[link],
            });
            
        }
        
        None
        
    }
    
}
