mod cache;
mod feed;

use std::{
    error::Error,
    fs::File,
    io::{ self, Read, Write },
    path::{ Path, PathBuf },
};

use cache::Cache;
use feed::Feed;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const CONTENT_SIZE_LIMIT: u64 = 1024 * 1024;

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
    
    let folder = rin::get(b"folder")?;
    let feeds = chiaki::List::load("feeds")?;
    let rules = chiaki::List::load("rules")?;
    
    // -------------------- cache --------------------
    
    let mut cache = Cache::new(&rules);
    
    // -------------------- client --------------------
    
    let mut client = akari::Client::new()?;
    
    // -------------------- entries --------------------
    
    for url in feeds.iter().filter_map(|feed| str::from_utf8(feed.tag).ok()) {
        
        println!();
        println!("{}", url);
        println!("--------------------");
        
        let feed = Feed::new(&mut client, url)?;
        
        for entry in &feed {
            
            // -------------------- rule --------------------
            
            // an update means the current feed entry is relevant
            // defer execution until torrent file has been downloaded
            let Some(rule_update) = cache.get_rule_update(entry.title) else {
                continue;
            };
            
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
            
            download_torrent(&mut client, link, &destination)?;
            
            rule_update.execute()?;
            
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
    
    if let Some(current) = file_path.extension() {
        if ! current.eq_ignore_ascii_case("torrent") {
            file_path.add_extension("torrent");
        }
    } else {
        file_path.set_extension("torrent");
    }
    
    Ok(file_path)
}

fn download_torrent(client: &mut akari::Client, link: &str, destination: &Path) -> Result<(), Box<dyn Error>> {
    let response = client.get(link)?;
    let mut file = File::options()
        .create_new(true)
        .write(true)
        .open(destination)?;
    
    let mut reader = chikuwa::LimitedReader::new(response, CONTENT_SIZE_LIMIT)?;
    
    io::copy(&mut reader, &mut file)?;
    
    Ok(())
}
