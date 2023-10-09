use std::{
    error::Error,
    ffi::OsString,
    fs,
    io::{ self, stdout, Write },
    path::Path,
    str,
};

struct Releases<'c, 'r> {
    content: &'c [u8],
    rules: &'r chiaki::List,
}

struct Release<'c, 'r> {
    matcher: &'r str,
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
    println!("Loading configuration file...");
    
    let config = rin::Config::load()?;
    let folder = config.get(b"folder")?;
    
    println!();
    println!("Loading feeds...");
    
    let feeds = chiaki::List::load("feeds")?;
    
    println!();
    println!("Loading rules...");
    
    let rules = chiaki::List::load("rules")?;
    
    println!();
    println!("Success!");
    
    let mut found: Vec<(&str, u64)> = Vec::with_capacity(20);
    
    for feed in feeds.iter() {
        
        println!();
        println!("{}", feed.tag);
        println!("--------------------");
        
        match akari::get(feed.tag) {
            
            Ok(source) => for release in Releases::from((source.as_slice(), &rules)) {
                
                if found.iter().any(|&(matcher, episode)| matcher == release.matcher && episode >= release.episode) {
                    continue;
                }
                
                println!("{}", release.title);
                
                match download_torrent(release.title, release.link, folder) {
                    Ok(()) => found.push((release.matcher, release.episode)),
                    Err(error) => println!("ERROR: {}", error),
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

fn download_torrent(title: &str, link: &str, folder: &str) -> Result<(), Box<dyn Error>> {
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
    
    if destination.exists() {
        return Err(chikuwa::concat_str!("File already exists: ", &destination.to_string_lossy()).into());
    }
    
    let content = akari::get(link)?;
    
    fs::write(destination, content)?;
    
    Ok(())
}

impl<'c, 'r> From<(&'c [u8], &'r chiaki::List)> for Releases<'c, 'r> {
    
    fn from(data: (&'c [u8], &'r chiaki::List)) -> Self {
        Self {
            content: data.0,
            rules: data.1,
        }
    }
    
}

impl<'c, 'r> Iterator for Releases<'c, 'r> {
    
    type Item = Release<'c, 'r>;
    
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

fn build_entry<'c, 'r>(item: &'c [u8], rules: &'r chiaki::List) -> Option<Release<'c, 'r>> {
    let title = chikuwa::tag_range(item, TITLE_OPEN_TAG, TITLE_CLOSE_TAG)
        .and_then(|field| str::from_utf8(&item[field]).ok())?;
    
    let rule = rules.iter().find(|rule| title.contains(rule.tag))?;
    
    let episode = extract_episode(title, rule.tag)
        .filter(|&episode| rule.value < episode)?;
    
    let link = chikuwa::tag_range(item, LINK_OPEN_TAG, LINK_CLOSE_TAG)
        .and_then(|field| str::from_utf8(&item[field]).ok())?;
    
    Some(Release {
        matcher: rule.tag,
        episode,
        title,
        link,
    })
}

fn extract_episode(title: &str, tag: &str) -> Option<u64> {
    let clean = title.replace(tag, "");
    let mut chars = clean.chars();
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
