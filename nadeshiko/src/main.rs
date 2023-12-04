mod releases;

use std::{
    error::Error,
    ffi::OsString,
    fs::{ self, File },
    io::{ self, Read, Write, BufWriter, },
    os::windows::io::{ AsRawHandle, FromRawHandle },
    path::Path,
    str,
};

use releases::Releases;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const FOUND_VEC_INITIAL_SIZE: usize = 20;
const TORRENT_FILE_WRITER_BUFFER_SIZE: usize = 64 * 1024;

fn main() {
    // avoid buffered output
    let mut stdout = unsafe {
        
        File::from_raw_handle(io::stdout().as_raw_handle())
        
    };
    
    stdout.write_all(format!("{} v{}", APP_NAME, APP_VERSION).as_bytes()).unwrap();
    
    if let Err(error) = process(&mut stdout) {
        stdout.write_all(format!("\n\n{}", error).as_bytes()).unwrap();
    }
    
    stdout.write_all(b"\n\nPress 'enter' key to exit...").unwrap();
    
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}

fn process(stdout: &mut File) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    stdout.write_all(b"\n\nLoading configuration...").unwrap();
    
    let folder = rin::get(b"folder")?;
    
    // -------------------- feeds --------------------
    
    stdout.write_all(b"\nLoading feeds...").unwrap();
    
    let feeds = chiaki::List::load("feeds")?;
    
    // -------------------- rules --------------------
    
    stdout.write_all(b"\nLoading rules...").unwrap();
    
    let rules = chiaki::List::load("rules")?;
    
    // -------------------- client --------------------
    
    let mut client = akari::Client::new()?;
    
    // -------------------- releases --------------------
    
    let mut found: Vec<(&[u8], u64)> = Vec::with_capacity(FOUND_VEC_INITIAL_SIZE);
    
    for url in feeds.iter().filter_map(|feed| str::from_utf8(feed.tag).ok()) {
        
        stdout.write_all(format!("\n\n{}", url).as_bytes()).unwrap();
        stdout.write_all(b"\n--------------------").unwrap();
        
        match client.get(url) {
            
            Ok(mut payload) => {
                
                let mut content = Vec::with_capacity(payload.content_length());
                
                if let Err(error) = payload.read_to_end(&mut content) {
                    stdout.write_all(format!("\nERROR: {}", error).as_bytes()).unwrap();
                    continue;
                }
                
                for release in Releases::new(&content, &rules) {
                    
                    if found.iter().any(|&(matcher, episode)| matcher == release.matcher && episode >= release.episode) {
                        continue;
                    }
                    
                    stdout.write_all(format!("\n{}", release.title).as_bytes()).unwrap();
                    
                    if let Err(error) = download_torrent(&mut client, release.title, release.link, folder) {
                        stdout.write_all(format!("\nERROR: {}", error).as_bytes()).unwrap();
                        continue;
                    }
                    
                    found.push((release.matcher, release.episode));
                    
                }
                
            },
            
            Err(error) => stdout.write_all(format!("\nERROR: {}", error).as_bytes()).unwrap(),
            
        }
        
    }
    
    // -------------------- commit --------------------
    
    if ! found.is_empty() {
        
        let mut rules = chiaki::List::load("rules")?;
        
        for (matcher, episode) in found {
            rules.update(matcher, episode)?;
        }
        
        rules.commit()?;
        
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
