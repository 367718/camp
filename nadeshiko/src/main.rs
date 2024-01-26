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

use releases::{ Releases, ReleasesEntry };

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const TORRENT_FILE_WRITER_BUFFER_SIZE: usize = 64 * 1024;

fn main() {
    // avoid locking
    let mut stdout = unsafe {
        
        File::from_raw_handle(io::stdout().as_raw_handle())
        
    };
    
    write!(&mut stdout, "{} v{}", APP_NAME, APP_VERSION).unwrap();
    
    if let Err(error) = process(&mut stdout) {
        write!(&mut stdout, "\n\n{}", error).unwrap();
    }
    
    stdout.write_all(b"\n\nPress 'enter' key to exit...").unwrap();
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}

fn process(stdout: &mut File) -> Result<(), Box<dyn Error>> {
    // -------------------- configuration --------------------
    
    stdout.write_all(b"\n\nLoading configuration...").unwrap();
    
    let folder = rin::get(b"folder")?;
    
    // -------------------- feeds --------------------
    
    stdout.write_all(b"\nLoading feeds...").unwrap();
    
    let feeds = chiaki::List::load("feeds")?;
    
    // -------------------- rules --------------------
    
    stdout.write_all(b"\nLoading rules...").unwrap();
    
    let mut rules = chiaki::List::load("rules")?;
    
    // -------------------- client --------------------
    
    let mut client = akari::Client::new()?;
    
    // -------------------- releases --------------------
    
    for url in feeds.iter().filter_map(|feed| str::from_utf8(feed.tag).ok()) {
        
        write!(stdout, "\n\n{}\n--------------------", url).unwrap();
        
        for release in Releases::new(&mut client, url)?.iter() {
            
            // -------------------- rule and episode --------------------
            
            let Some(rule) = rules.iter().find(|rule| release.title.starts_with(rule.tag)) else {
                continue;
            };
            
            let Some(episode) = extract_episode(&release, &rule) else {
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
            
            write!(stdout, "\n{}", title).unwrap();
            
            download_torrent(&mut client, title, link, folder)?;
            
            // used release title instead of rule tag to avoid borrowing error
            rules.update(&release.title[..rule.tag.len()], episode)?;
            
        }
        
    }
    
    Ok(())
}

fn extract_episode(release: &ReleasesEntry, rule: &chiaki::ListEntry) -> Option<u64> {
    let clean = &release.title[rule.tag.len()..];
    let mut chars = clean.iter().copied().map(char::from);
    let mut episode = chars.find_map(|current| current.to_digit(10).map(u64::from))?;
    
    while let Some(digit) = chars.next().and_then(|current| current.to_digit(10).map(u64::from)) {
        episode = episode.checked_mul(10)?.checked_add(digit)?;
    }
    
    Some(episode)
}

fn download_torrent(client: &mut akari::Client, title: &str, link: &str, folder: &str) -> Result<(), Box<dyn Error>> {
    let mut payload = client.get(link)?;
    
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
    
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    
    let mut writer = BufWriter::with_capacity(TORRENT_FILE_WRITER_BUFFER_SIZE, file);
    
    io::copy(&mut payload, &mut writer)?;
    
    writer.flush()?;
    
    Ok(())
}
