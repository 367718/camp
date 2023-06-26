use std::{
    ops::Range,
    str,
};

use crate::IsCandidate;

pub struct DownloadsEntries<'f, T> {
    feed: &'f [u8],
    candidates: &'f [T],
}

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DownloadsEntry<'f> {
    pub title: &'f str,
    pub link: &'f str,
    pub episode: i64,
    pub id: i64,
}

const ITEM_OPEN_TAG: &[u8] = b"<item>";
const ITEM_CLOSE_TAG: &[u8] = b"</item>";
const TITLE_OPEN_TAG: &[u8] = b"<title>";
const TITLE_CLOSE_TAG: &[u8] = b"</title>";
const LINK_OPEN_TAG: &[u8] = b"<link>";
const LINK_CLOSE_TAG: &[u8] = b"</link>";

impl<'f, T: IsCandidate> DownloadsEntries<'f, T> {
    
    pub fn get(feed: &'f [u8], candidates: &'f [T]) -> Self {
        Self {
            feed,
            candidates,
        }
    }
    
}

impl<'f, T: IsCandidate> Iterator for DownloadsEntries<'f, T> {
    
    type Item = DownloadsEntry<'f>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.feed.is_empty() {
            return None;
        }
        
        while let Some(item) = get_tag_range(self.feed, ITEM_OPEN_TAG, ITEM_CLOSE_TAG) {
            
            let result = build_entry(&self.feed[item.start..item.end], self.candidates);
            self.feed = &self.feed[item.end.saturating_add(ITEM_CLOSE_TAG.len())..];
            
            if result.is_some() {
                return result;
            }
            
        }
        
        None
    }
    
}

fn build_entry<'f, T: IsCandidate>(item: &'f [u8], candidates: &'f [T]) -> Option<DownloadsEntry<'f>> {
    let title = get_tag_range(item, TITLE_OPEN_TAG, TITLE_CLOSE_TAG)
        .and_then(|field| str::from_utf8(&item[field]).ok())
        .map(str::trim)?;
    
    let candidate = candidates.iter()
        .find(|candidate| candidate.is_relevant(title))?;
    
    let episode = crate::extractor::get(&candidate.clean(title))
        .filter(|&episode| candidate.can_download(episode))?;
    
    let link = get_tag_range(item, LINK_OPEN_TAG, LINK_CLOSE_TAG)
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
        id: i64,
    }
    
    impl IsCandidate for CandidatesEntry {
        
        fn is_relevant(&self, current: &str) -> bool {
            current.contains(&self.title)
        }
        
        fn clean(&self, current: &str) -> String {
            current.replace(&self.title, "")
        }
        
        fn can_download(&self, _episode: i64) -> bool {
            true
        }
        
        fn can_update(&self, _episode: i64) -> bool {
            true
        }
        
        fn id(&self) -> i64 {
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
            
            let output = DownloadsEntries::get(&feed, &candidates);
            
            // control
            
            let output: Vec<DownloadsEntry> = output.collect();
            
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
            
            let output = DownloadsEntries::get(&feed, &candidates);
            
            // control
            
            let output: Vec<DownloadsEntry> = output.collect();
            
            assert!(output.is_empty());
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
            write!(feed, "<title>[Invalid] Undefined - 2 [720p]</title>").unwrap();
            write!(feed, "<link>http://example.com/undefined</link>").unwrap();
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
