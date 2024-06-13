use std::{
    error::Error,
    ffi::OsString,
    fs,
    io::{ self, Read, Write },
    path::{ Path, PathBuf },
    str,
};

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
    
    io::stdout().flush().unwrap();
    let _ = io::stdin().read(&mut [0]).unwrap();
}

fn process() -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    println!();
    println!("Loading configuration...");
    
    let folder = rin::get(b"folder")?;
    
    // -------------------- feeds --------------------
    
    println!("Loading feeds...");
    
    let feeds = chiaki::List::load("feeds")?;
    
    // -------------------- rules --------------------
    
    println!("Loading rules...");
    
    let mut rules = chiaki::List::load("rules")?;
    
    // -------------------- client --------------------
    
    let mut client = akari::Client::new()?;
    
    // -------------------- entries --------------------
    
    for url in feeds.iter().filter_map(|feed| str::from_utf8(feed.tag).ok()) {
        
        println!();
        println!("{}", url);
        println!("--------------------");
        
        for entry in chikuwa::RssFeed::new(&get_feed_content(&mut client, url)?) {
            
            // -------------------- rule and episode --------------------
            
            let Some(rule) = rules.iter().find(|rule| entry.title.starts_with(rule.tag)) else {
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
            
            // since the list update can fail, an ephemeral path is used to prevent leaving a torrent file existing in destination for a future run
            let destination = chikuwa::EphemeralPath::from(build_destination(folder, title)?);
            
            download_torrent(&mut client, link, &destination)?;
            
            // entry title used instead of rule tag to avoid borrowing error
            rules.update(&entry.title[..rule.tag.len()], episode)?;
            
            destination.make_permanent();
            
        }
        
    }
    
    Ok(())
}

fn get_feed_content(client: &mut akari::Client, url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut response = client.get(url)?;
    
    let mut content = Vec::with_capacity(response.content_length());
    response.read_to_end(&mut content)?;
    
    Ok(content)
}

fn build_destination(folder: &str, title: &str) -> Result<PathBuf, Box<dyn Error>> {
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
    
    Ok(destination)
}

fn download_torrent(client: &mut akari::Client, link: &str, destination: &Path) -> Result<(), Box<dyn Error>> {
    let mut response = client.get(link)?;
    
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    
    io::copy(&mut response, &mut file)?;
    
    Ok(())
}
