use std::{
    error::Error,
    io::Write,
    path::{ Path, PathBuf },
    process::{ Command, Stdio },
    str,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/files.html");

pub enum FilesEndpoint {
    Index,
    Entries,
    Play,
    Mark,
    Move,
    Delete,
}

impl FilesEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/files/") => Some(Self::Index),
            (b"GET", b"/files/entries") => Some(Self::Entries),
            (b"POST", b"/files/play") => Some(Self::Play),
            (b"POST", b"/files/mark") => Some(Self::Mark),
            (b"POST", b"/files/move") => Some(Self::Move),
            (b"POST", b"/files/delete") => Some(Self::Delete),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Entries => entries(&mut request),
            Self::Play => play(&mut request),
            Self::Mark => mark(&mut request),
            Self::Move => move_to_folder(&mut request),
            Self::Delete => delete(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    let root = rin::get(b"root")?;
    let flag = rin::get(b"flag")?;
    
    // -------------------- list --------------------
    
    let files = ena::Files::new(PathBuf::from(root));
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in files {
        
        write!(&mut response, "<a tabindex='0' data-value='{}'>", entry.value(flag))?;
        
        if let Some(container) = entry.container(root) {
            response.write_all(b"<span>")?;
            response.write_all(container.as_bytes())?;
            response.write_all(b"</span>")?;
        }
        
        response.write_all(entry.name().as_bytes())?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    let root = rin::get(b"root")?;
    let player = rin::get(b"player")?;
    
    // -------------------- files --------------------
    
    let mut files = request.param(b"tag")
        .filter_map(|path| str::from_utf8(path).ok())
        .map(|path| Path::new(root).join(path))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- operation --------------------
    
    Command::new(player)
        .args(files)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn mark(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    let root = rin::get(b"root")?;
    let flag = rin::get(b"flag")?;
    
    // -------------------- files --------------------
    
    let mut files = request.param(b"tag")
        .filter_map(|path| str::from_utf8(path).ok())
        .map(|path| Path::new(root).join(path))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- operation --------------------
    
    for mut entry in files {
        entry.mark(flag)?;
    }
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn move_to_folder(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    let root = rin::get(b"root")?;
    
    // -------------------- files --------------------
    
    let mut files = request.param(b"tag")
        .filter_map(|path| str::from_utf8(path).ok())
        .map(|path| Path::new(root).join(path))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- name --------------------
    
    let name = request.param(b"input")
        .next()
        .and_then(|path| str::from_utf8(path).ok())
        .ok_or("Invalid name")?;
    
    // -------------------- operation --------------------
    
    for entry in files {
        entry.move_to_folder(root, name)?;
    }
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    let root = rin::get(b"root")?;
    
    // -------------------- files --------------------
    
    let mut files = request.param(b"tag")
        .filter_map(|path| str::from_utf8(path).ok())
        .map(|path| Path::new(root).join(path))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File not provided".into());
    }
    
    // -------------------- operation --------------------
    
    for entry in files {
        entry.delete()?;
    }
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}
