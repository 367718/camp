mod releases;
mod extractor;

use std::{
    error::Error,
    ffi::OsString,
    fs,
    io::{ self, Read, Write, BufWriter, },
    path::{ Path, PathBuf },
    str,
};

use releases::Releases;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const TORRENT_FILE_WRITER_BUFFER_SIZE: usize = 64 * 1024;

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
    
    // -------------------- releases --------------------
    
    for url in feeds.iter().filter_map(|feed| str::from_utf8(feed.tag).ok()) {
        
        println!();
        println!("{}", url);
        println!("--------------------");
        
        for release in Releases::new(&get_content(&mut client, url)?) {
            
            // -------------------- rule and episode --------------------
            
            let Some(rule) = rules.iter().find(|rule| release.title.starts_with(rule.tag)) else {
                continue;
            };
            
            let Some(episode) = extractor::get_episode(&release.title[rule.tag.len()..]) else {
                continue;
            };
            
            // -------------------- relevant --------------------
            
            if episode <= rule.value {
                continue;
            }
            
            // -------------------- fields --------------------
            
            let Ok(title) = str::from_utf8(release.title) else {
                continue;
            };
            
            let Ok(link) = str::from_utf8(release.link) else {
                continue;
            };
            
            // -------------------- download and update --------------------
            
            println!("{}", title);
            
            // since the list update can fail, an ephemeral path is used to prevent leaving a torrent file existing in destination for a future run
            let destination = chikuwa::EphemeralPath::from(build_destination(folder, title)?);
            
            download_torrent(&mut client, link, &destination)?;
            
            // release title used instead of rule tag to avoid borrowing error
            rules.update(&release.title[..rule.tag.len()], episode)?;
            
            destination.make_permanent();
            
        }
        
    }
    
    Ok(())
}

fn get_content(client: &mut akari::Client, url: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut payload = client.get(url)?;
    
    let mut content = Vec::with_capacity(payload.content_length());
    payload.read_to_end(&mut content)?;
    
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
    let mut payload = client.get(link)?;
    
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    
    let mut writer = BufWriter::with_capacity(TORRENT_FILE_WRITER_BUFFER_SIZE, file);
    
    io::copy(&mut payload, &mut writer)?;
    
    writer.flush()?;
    
    Ok(())
}
