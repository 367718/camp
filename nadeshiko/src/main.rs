mod feed;

use std::{
    error::Error,
    ffi::OsStr,
    fs::File,
    io::{ self, Read, Write, BufWriter },
    path::{ Path, PathBuf },
};

use feed::Feed;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);
    
    if let Err(error) = process() {
        println!();
        println!("ERROR: {}", error);
    }
    
    println!();
    print!("Press 'enter' key to exit...");
    io::stdout().flush().ok();
    
    let _ = io::stdin().read(&mut [0]).ok();
}

fn process() -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let folder = rin::get::<&str>(b"folder")?;
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    let max_feed_size = rin::get::<u64>(b"max_feed_size")?;
    let max_torrent_size = rin::get::<u64>(b"max_torrent_size")?;
    
    let feeds = chiaki::List::load("feeds", max_list_size)?;
    let rules = chiaki::List::load("rules", max_list_size)?;
    
    // -------------------- cache --------------------
    
    let mut cache = rules.iter().collect::<Vec<chiaki::ListEntry>>();
    
    // -------------------- client --------------------
    
    let mut client = akari::Client::new()?;
    
    // -------------------- entries --------------------
    
    for feed in &feeds {
        
        let url = str::from_utf8(feed.tag)?;
        
        println!();
        println!("{}", url);
        println!("--------------------");
        
        for entry in &Feed::new(&mut client, url, max_feed_size)? {
            
            // -------------------- rule and episode --------------------
            
            let Some(rule) = cache.iter_mut().find(|rule| entry.title.starts_with(rule.tag)) else {
                continue;
            };
            
            let Some(episode) = chikuwa::first_number(&entry.title[rule.tag.len()..]) else {
                continue;
            };
            
            if rule.value >= episode {
                continue;
            }
            
            // -------------------- conversion --------------------
            
            let Ok(title) = str::from_utf8(entry.title) else {
                continue;
            };
            
            let Ok(link) = str::from_utf8(entry.link) else {
                continue;
            };
            
            // -------------------- download and update --------------------
            
            println!("{}", title);
            
            // rule update may fail, so torrent path is initially treated as ephemeral
            let destination = chikuwa::EphemeralPath::from(build_destination(folder, title)?);
            
            download(&mut client, link, max_torrent_size, &destination)?;
            update(rule, episode, max_list_size)?;
            
            destination.make_permanent();
            
        }
        
    }
    
    Ok(())
}

fn build_destination(folder: &str, title: &str) -> Result<PathBuf, Box<dyn Error>> {
    let file_name = chikuwa::win_filename(title)
        .ok_or(format!("Invalid file name for torrent file: {}", title))?;
    
    let mut file_path = Path::new(folder)
        .join(file_name);
    
    let current_extension = file_path.extension()
        .unwrap_or_else(|| OsStr::new(""));
    
    if ! current_extension.eq_ignore_ascii_case("torrent") {
        file_path.add_extension("torrent");
    }
    
    Ok(file_path)
}

fn download(client: &mut akari::Client, link: &str, max_size: u64, destination: &Path) -> Result<(), Box<dyn Error>> {
    let response = client.get(link)?;
    
    let file = File::options()
        .create_new(true)
        .write(true)
        .open(destination)?;
    
    let mut reader = response.take(max_size);
    let mut writer = BufWriter::new(file);
    
    io::copy(&mut reader, &mut writer)?;
    
    writer.flush()?;
    
    Ok(())
}

fn update(rule: &mut chiaki::ListEntry, episode: u16, max_size: u64) -> Result<(), Box<dyn Error>> {
    chiaki::List::load("rules", max_size)
        .and_then(|list| list.set(rule.tag, episode))?;
    
    rule.value = episode;
    
    Ok(())
}
