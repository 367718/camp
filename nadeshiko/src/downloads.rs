use std::str;

use chiaki::CandidatesEntry;

pub struct DownloadsEntries<'f, 'c> {
    feed: &'f [u8],
    candidates: &'c [&'c CandidatesEntry],
}

#[derive(PartialEq, Eq)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct DownloadsEntry<'f, 'c> {
    pub title: &'f str,
    pub link: &'f str,
    pub episode: i64,
    pub candidate: &'c CandidatesEntry,
}

const ITEM_OPEN_TAG: &[u8] = b"<item>";
const ITEM_CLOSE_TAG: &[u8] = b"</item>";
const TITLE_OPEN_TAG: &[u8] = b"<title>";
const TITLE_CLOSE_TAG: &[u8] = b"</title>";
const LINK_OPEN_TAG: &[u8] = b"<link>";
const LINK_CLOSE_TAG: &[u8] = b"</link>";

impl<'f, 'c> DownloadsEntries<'f, 'c> {
    
    pub fn get(feed: &'f [u8], candidates: &'c [&'c CandidatesEntry]) -> Self {
        Self {
            feed,
            candidates,
        }
    }
    
}

impl<'f, 'c> Iterator for DownloadsEntries<'f, 'c> {
    
    type Item = DownloadsEntry<'f, 'c>;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(range) = chikuwa::tag_range(self.feed, ITEM_OPEN_TAG, ITEM_CLOSE_TAG) {
            
            let entry = build_entry(&self.feed[range.start..range.end], self.candidates);
            self.feed = &self.feed[range.end..][ITEM_CLOSE_TAG.len()..];
            
            if entry.is_some() {
                return entry;
            }
            
        }
        
        None
    }
    
}

fn build_entry<'f, 'c>(item: &'f [u8], candidates: &'c [&'c CandidatesEntry]) -> Option<DownloadsEntry<'f, 'c>> {
    let title = chikuwa::tag_range(item, TITLE_OPEN_TAG, TITLE_CLOSE_TAG)
        .and_then(|field| str::from_utf8(&item[field]).ok())?;
    
    let candidate = candidates.iter()
        .find(|candidate| candidate.pieces().iter().all(|piece| title.contains(piece)))?;
    
    let episode = crate::extractor::get(title, &candidate.pieces())
        .filter(|episode| ! candidate.downloaded().contains(episode))?;
    
    let link = chikuwa::tag_range(item, LINK_OPEN_TAG, LINK_CLOSE_TAG)
        .and_then(|field| str::from_utf8(&item[field]).ok())?;
    
    Some(DownloadsEntry {
        title,
        link,
        episode,
        candidate,
    })
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let feed = generate_feed();
            
            let candidates = [
                &CandidatesEntry::new().with_title(String::from("Fictional")),
                &CandidatesEntry::new().with_title(String::from("Not defined")),
                &CandidatesEntry::new().with_title(String::from("Test")),
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
                    candidate: &CandidatesEntry::new().with_title(String::from("Fictional")),
                },
                DownloadsEntry {
                    title: "test/[Placeholder] Test - 10 [1080p]",
                    link: "http://example.com/releases/564683.torrent",
                    episode: 10,
                    candidate: &CandidatesEntry::new().with_title(String::from("Test")),
                },
                DownloadsEntry {
                    title: "[Placeholder] Test - 11 [1080p]",
                    link: "http://example.com/releases/8723034.torrent",
                    episode: 11,
                    candidate: &CandidatesEntry::new().with_title(String::from("Test")),
                },
                DownloadsEntry {
                    title: "[Placeholder] Test - 12 [1080p]",
                    link: "http://example.com/releases/7821023.torrent",
                    episode: 12,
                    candidate: &CandidatesEntry::new().with_title(String::from("Test")),
                },
            ]));
        }
        
        #[test]
        fn invalid() {
            // setup
            
            let feed = generate_feed();
            
            let candidates = [
                &CandidatesEntry::new().with_title(String::from("Not defined")),
            ];
            
            // operation
            
            let output = DownloadsEntries::get(&feed, &candidates);
            
            // control
            
            let output: Vec<DownloadsEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        #[test]
        fn empty() {
            // setup
            
            let feed = Vec::new();
            
            let candidates = Vec::new();
            
            // operation
            
            let output = DownloadsEntries::get(&feed, &candidates);
            
            // control
            
            let output: Vec<DownloadsEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        #[test]
        fn empty_feed() {
            // setup
            
            let feed = Vec::new();
            
            let candidates = [
                &CandidatesEntry::new().with_title(String::from("Fictional")),
                &CandidatesEntry::new().with_title(String::from("Not defined")),
                &CandidatesEntry::new().with_title(String::from("Test")),
            ];
            
            // operation
            
            let output = DownloadsEntries::get(&feed, &candidates);
            
            // control
            
            let output: Vec<DownloadsEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        #[test]
        fn empty_candidates() {
            // setup
            
            let feed = generate_feed();
            
            let candidates = Vec::new();
            
            // operation
            
            let output = DownloadsEntries::get(&feed, &candidates);
            
            // control
            
            let output: Vec<DownloadsEntry> = output.collect();
            
            assert!(output.is_empty());
        }
        
        fn generate_feed() -> Vec<u8> {
            let mut feed = Vec::new();
            
            feed.extend_from_slice(b"<rss>");
            feed.extend_from_slice(b"<channel>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<title>[Imaginary] Fictional - 10 [480p]</title>");
            feed.extend_from_slice(b"<link>http://example.com/invalid</link>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<title>[Invalid] Undefined - 2 [720p]</title>");
            feed.extend_from_slice(b"<link>http://example.com/undefined</link>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<title>test/[Placeholder] Test - 10 [1080p]</title>");
            feed.extend_from_slice(b"<link>http://example.com/releases/564683.torrent</link>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<title>[Placeholder] Test - 11 [1080p]</title>");
            feed.extend_from_slice(b"<link>http://example.com/releases/8723034.torrent</link>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<title>[Placeholder] Test - 12 [1080p]</title>");
            feed.extend_from_slice(b"<link>http://example.com/releases/7821023.torrent</link>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<title>title</title>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"<item>");
            feed.extend_from_slice(b"<link>link</link>");
            feed.extend_from_slice(b"</item>");
            
            feed.extend_from_slice(b"</channel>");
            feed.extend_from_slice(b"</rss>");
            
            feed
        }
        
    }
    
}
