use std::str;

pub struct Releases<'c, 'r> {
    content: &'c [u8],
    rules: &'r chiaki::List,
}

pub struct ReleasesEntry<'c, 'r> {
    pub matcher: &'r [u8],
    pub episode: u64,
    pub title: &'c str,
    pub link: &'c str,
}

const ITEM_OPEN_TAG: &[u8] = b"<item>";
const ITEM_CLOSE_TAG: &[u8] = b"</item>";
const TITLE_OPEN_TAG: &[u8] = b"<title>";
const TITLE_CLOSE_TAG: &[u8] = b"</title>";
const LINK_OPEN_TAG: &[u8] = b"<link>";
const LINK_CLOSE_TAG: &[u8] = b"</link>";

impl<'c, 'r> Releases<'c, 'r> {
    
    pub fn new(content: &'c [u8], rules: &'r chiaki::List) -> Self {
        Self {
            content,
            rules,
        }
    }
    
}

impl<'c, 'r> Iterator for Releases<'c, 'r> {
    
    type Item = ReleasesEntry<'c, 'r>;
    
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
        
        while let Some(range) = chikuwa::tag_range(self.content, ITEM_OPEN_TAG, ITEM_CLOSE_TAG) {
            
            let item = &self.content[range.start..range.end];
            let entry = build_entry(item, self.rules);
            
            self.content = &self.content[range.end..][ITEM_CLOSE_TAG.len()..];
            
            if entry.is_some() {
                return entry;
            }
            
        }
        
        None
        
    }
    
}

fn build_entry<'c, 'r>(item: &'c [u8], rules: &'r chiaki::List) -> Option<ReleasesEntry<'c, 'r>> {
    let title = chikuwa::tag_range(item, TITLE_OPEN_TAG, TITLE_CLOSE_TAG)
        .map(|field| &item[field])?;
    
    let rule = rules.iter().find(|rule| title.starts_with(rule.tag))?;
    
    let episode = extract_episode(&title[rule.tag.len()..])
        .filter(|&episode| rule.value < episode)?;
    
    let link = chikuwa::tag_range(item, LINK_OPEN_TAG, LINK_CLOSE_TAG)
        .map(|field| &item[field])?;
    
    Some(ReleasesEntry {
        matcher: rule.tag,
        episode,
        title: str::from_utf8(title).ok()?,
        link: str::from_utf8(link).ok()?,
    })
}

fn extract_episode(title: &[u8]) -> Option<u64> {
    let mut chars = title.iter().copied().map(char::from);
    let mut result = chars.find_map(|current| current.to_digit(10)).map(u64::from)?;
    
    while let Some(current) = chars.next() {
        
        if let Some(digit) = current.to_digit(10).map(u64::from) {
            result = result.checked_mul(10)?.checked_add(digit)?;
            continue;
        }
        
        // if next to a digit is a dot and next to the dot is another digit, abort
        if current == '.' && chars.next().filter(char::is_ascii_digit).is_some() {
            return None;
        }
        
        break;
        
    }
    
    Some(result)
}
