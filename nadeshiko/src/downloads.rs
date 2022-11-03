use std::{
    ops::Range,
    str,
};

use crate::IsCandidate;

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DownloadsEntry<'f> {
    pub title: &'f str,
    pub link: &'f str,
    pub episode: u32,
    pub id: u32,
}

const RESULT_VEC_INITIAL_CAPACITY: usize = 20;

pub fn get<'f>(feed: &'f [u8], candidates: &[impl IsCandidate]) -> Option<Vec<DownloadsEntry<'f>>> {
    let mut result = Vec::with_capacity(RESULT_VEC_INITIAL_CAPACITY);
    
    let mut content = feed;
    
    while let Some(item) = get_tag_range(content, b"<item>", b"</item>") {
        
        if let Some(entry) = build_entry(&content[item.start..item.end], candidates) {
            result.push(entry);
        }
        
        let Some(start) = item.end.checked_add(b"</item>".len()) else {
            break;
        };
        
        content = &content[start..];
        
    }
    
    if ! result.is_empty() {
        return Some(result);
    }
    
    None
}

fn build_entry<'f>(item: &'f [u8], candidates: &[impl IsCandidate]) -> Option<DownloadsEntry<'f>> {
    let title = get_tag_range(item, b"<title>", b"</title>")
        .and_then(|field| str::from_utf8(&item[field]).ok())
        .map(str::trim)?;
    
    let candidate = candidates.iter()
        .find(|candidate| candidate.is_relevant(title))?;
    
    let episode = crate::extractor::get(&candidate.clean(title))
        .filter(|&episode| candidate.can_download(episode))?;
    
    let link = get_tag_range(item, b"<link>", b"</link>")
        .and_then(|field| str::from_utf8(&item[field]).ok())
        .map(str::trim)?;
    
    let id = candidate.id();
    
    Some(DownloadsEntry {
        title,
        link,
        episode,
        id,
    })
}

fn get_tag_range(content: &[u8], open: &[u8], close: &[u8]) -> Option<Range<usize>> {
    let start = content.windows(open.len())
        .position(|window| window == open)
        .and_then(|index| index.checked_add(open.len()))?;
    
    let end = content[start..].windows(close.len())
        .position(|window| window == close)
        .and_then(|index| index.checked_add(start))?;
    
    Some(start..end)
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    struct CandidatesEntry {
        title: String,
        id: u32,
    }
    
    impl IsCandidate for CandidatesEntry {
        
        fn is_relevant(&self, current: &str) -> bool {
            current.contains(&self.title)
        }
        
        fn clean(&self, current: &str) -> String {
            current.replace(&self.title, "")
        }
        
        fn can_download(&self, _episode: u32) -> bool {
            true
        }
        
        fn can_update(&self, _episode: u32) -> bool {
            true
        }
        
        fn id(&self) -> u32 {
            self.id
        }
        
    }
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        use std::io::Write;
        
        #[test]
        fn valid() {
            // setup
            
            let feed = generate_feed();
            
            let candidates = [
                CandidatesEntry {
                    title: String::from("Fictional"),
                    id: 15,
                },
                CandidatesEntry {
                    title: String::from("Not defined"),
                    id: 2,
                },
                CandidatesEntry {
                    title: String::from("Test"),
                    id: 10,
                },
            ];
            
            // operation
            
            let output = get(&feed, &candidates);
            
            // control
            
            assert!(output.is_some());
            
            let output = output.unwrap();
            
            assert_eq!(output, Vec::from([
                DownloadsEntry {
                    title: "[Imaginary] Fictional - 10 [480p]",
                    link: "http://example.com/invalid",
                    episode: 10,
                    id: 15,
                },
                DownloadsEntry {
                    title: "test/[Placeholder] Test - 10 [1080p]",
                    link: "http://example.com/releases/564683.torrent",
                    episode: 10,
                    id: 10,
                },
                DownloadsEntry {
                    title: "[Placeholder] Test - 11 [1080p]",
                    link: "http://example.com/releases/8723034.torrent",
                    episode: 11,
                    id: 10,
                },
                DownloadsEntry {
                    title: "[Placeholder] Test - 12 [1080p]",
                    link: "http://example.com/releases/7821023.torrent",
                    episode: 12,
                    id: 10,
                },
            ]));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let feed = generate_feed();
            
            let candidates = [
                CandidatesEntry {
                    title: String::from("Not defined"),
                    id: 2,
                },
            ];
            
            // operation
            
            let output = get(&feed, &candidates);
            
            // control
            
            assert!(output.is_none());
        }
        
        fn generate_feed() -> Vec<u8> {
            let mut feed = Vec::new();
            
            write!(feed, "<rss>").unwrap();
            write!(feed, "<channel>").unwrap();
            
            write!(feed, "<item>").unwrap();
            write!(feed, "<title>[Imaginary] Fictional - 10 [480p]</title>").unwrap();
            write!(feed, "<link>http://example.com/invalid</link>").unwrap();
            write!(feed, "</item>").unwrap();
            
            write!(feed, "<item>").unwrap();
            write!(feed, "<item>").unwrap();
            write!(feed, "<item>").unwrap();
            write!(feed, "<title>test/[Placeholder] Test - 10 [1080p]</title>").unwrap();
            write!(feed, "<link>http://example.com/releases/564683.torrent</link>").unwrap();
            write!(feed, "</item>").unwrap();
            
            write!(feed, "<item>").unwrap();
            write!(feed, "<title>[Placeholder] Test - 11 [1080p]</title>").unwrap();
            write!(feed, "<link>http://example.com/releases/8723034.torrent</link>").unwrap();
            write!(feed, "</item>").unwrap();
            
            write!(feed, "<item>").unwrap();
            write!(feed, "<title>[Placeholder] Test - 12 [1080p]</title>").unwrap();
            write!(feed, "<link>http://example.com/releases/7821023.torrent</link>").unwrap();
            write!(feed, "</item>").unwrap();
            
            write!(feed, "<item>").unwrap();
            write!(feed, "<title>title</title>").unwrap();
            write!(feed, "</item>").unwrap();
            
            write!(feed, "<item>").unwrap();
            write!(feed, "<link>link</link>").unwrap();
            write!(feed, "</item>").unwrap();
            
            write!(feed, "</channel>").unwrap();
            write!(feed, "</rss>").unwrap();
            
            feed
        }
        
    }
    
}
