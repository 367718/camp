mod rules;
mod feed;

use std::{
    error::Error,
    ffi::OsString,
    fs::File,
    io::{ self, Read, Write },
    path::{ Path, PathBuf },
};

use rules::Rules;
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
    
    io::stdout().flush().unwrap();
    let _ = io::stdin().read(&mut [0]).unwrap();
}

fn process() -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let folder = rin::get(b"folder")?;
    let feeds = chiaki::List::load("feeds")?;
    
    let mut rules = Rules::load()?;
    
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
            
            // defer update until torrent file has been downloaded
            let Some(rule_update) = rules.get_update(entry.title) else {
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
    let filename = chikuwa::win_filename(title).ok_or("Invalid file name")?;
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
    
    Ok(destination)
}

fn download_torrent(client: &mut akari::Client, link: &str, destination: &Path) -> Result<(), Box<dyn Error>> {
    let response = client.get(link)?;
    let mut file = File::options()
        .create_new(true)
        .write(true)
        .open(destination)?;
    
    let mut handle = response.take(CONTENT_SIZE_LIMIT);
    
    io::copy(&mut handle, &mut file)?;
    
    Ok(())
}
