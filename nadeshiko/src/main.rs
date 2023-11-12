use std::{
    error::Error,
    ffi::OsString,
    fs,
    io::{ self, stdout, Read, Write, BufWriter, },
    path::Path,
    str,
};

struct Releases<'c, 'r> {
    content: &'c [u8],
    rules: &'r chiaki::List,
}

struct ReleasesEntry<'c, 'r> {
    matcher: &'r [u8],
    episode: u64,
    title: &'c str,
    link: &'c str,
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const ITEM_OPEN_TAG: &[u8] = b"<item>";
const ITEM_CLOSE_TAG: &[u8] = b"</item>";
const TITLE_OPEN_TAG: &[u8] = b"<title>";
const TITLE_CLOSE_TAG: &[u8] = b"</title>";
const LINK_OPEN_TAG: &[u8] = b"<link>";
const LINK_CLOSE_TAG: &[u8] = b"</link>";

const TORRENT_FILE_WRITER_BUFFER_SIZE: usize = 64 * 1024;

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);
    println!("--------------------");
    
    if let Err(error) = process() {
        println!();
        println!("ERROR: {}", error);
    }
    
    println!();
    print!("Press 'enter' key to exit...");
    
    stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn process() -> Result<(), Box<dyn Error>> {
    println!();
    println!("Loading configuration...");
    
    let config = rin::Config::load()?;
    let folder = config.get(b"folder")?;
    
    println!("Loading feeds...");
    
    let feeds = chiaki::List::load("feeds")?;
    
    println!("Loading rules...");
    
    let rules = chiaki::List::load("rules")?;
    
    println!("Success!");
    
    let mut client = akari::Client::new()?;
    let mut found: Vec<(&[u8], u64)> = Vec::with_capacity(20);
    
    for url in feeds.iter().filter_map(|feed| str::from_utf8(feed.tag).ok()) {
        
        println!();
        println!("{}", url);
        println!("--------------------");
        
        match client.get(url) {
            
            Ok(mut payload) => {
                
                let mut content = Vec::with_capacity(payload.content_length());
                
                if let Err(error) = payload.read_to_end(&mut content) {
                    println!("ERROR: {}", error);
                    continue;
                }
                
                for release in Releases::new(&content, &rules) {
                    
                    if found.iter().any(|&(matcher, episode)| matcher == release.matcher && episode >= release.episode) {
                        continue;
                    }
                    
                    println!("{}", release.title);
                    
                    if let Err(error) = download_torrent(&mut client, release.title, release.link, folder) {
                        println!("ERROR: {}", error);
                        continue;
                    }
                    
                    found.push((release.matcher, release.episode));
                    
                }
                
            },
            
            Err(error) => println!("ERROR: {}", error),
            
        }
        
    }
    
    if ! found.is_empty() {
        
        let mut rules = chiaki::List::load("rules")?;
        
        for (matcher, episode) in found {
            rules.update(matcher, episode)?;
        }
        
    }
    
    Ok(())
}

fn download_torrent(client: &mut akari::Client, title: &str, link: &str, folder: &str) -> Result<(), Box<dyn Error>> {
    let filename = Path::new(title).file_name().ok_or("Invalid file name")?;
    let mut destination = Path::new(folder).join(filename);
    
    if let Some(current) = destination.extension() {
        if ! current.eq_ignore_ascii_case("torrent") {
            let mut composite = OsString::with_capacity(current.len() + 8);
            composite.push(current);
            composite.push(".torrent");
            destination.set_extension(composite);
        }
    } else {
        destination.set_extension("torrent");
    }
    
    let file = fs::OpenOptions::new().write(true)
        .create_new(true)
        .open(destination)?;
    
    let mut writer = BufWriter::with_capacity(TORRENT_FILE_WRITER_BUFFER_SIZE, file);
    
    io::copy(&mut client.get(link)?, &mut writer)?;
    
    writer.flush()?;
    
    Ok(())
}

impl<'c, 'r> Releases<'c, 'r> {
    
    fn new(content: &'c [u8], rules: &'r chiaki::List) -> Self {
        Self {
            content,
            rules,
        }
    }
    
}

impl<'c, 'r> Iterator for Releases<'c, 'r> {
    
    type Item = ReleasesEntry<'c, 'r>;
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(range) = chikuwa::tag_range(self.content, ITEM_OPEN_TAG, ITEM_CLOSE_TAG) {
            
            let entry = build_entry(&self.content[range.start..range.end], self.rules);
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
