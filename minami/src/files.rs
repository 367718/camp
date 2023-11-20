use std::{
    error::Error,
    path::{ MAIN_SEPARATOR_STR, Path, PathBuf },
    process::Command,
    str,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

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
                .and_then(|mut response| response.send(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)?;
    
    response.send(b"<!DOCTYPE html>")?;
    response.send(b"<html lang='en'>")?;
    
    // ---------- head ----------
    
    {
        
        response.send(b"<head>")?;
        
        response.send(b"<meta charset='utf-8'>")?;
        response.send(b"<meta name='viewport' content='width=device-width, initial-scale=1'>")?;
        response.send(b"<title>minami</title>")?;
        response.send(b"<link rel='icon' type='image/x-icon' href='/general/favicon.ico'>")?;
        response.send(b"<link rel='stylesheet' type='text/css' href='/general/styles.css'>")?;
        response.send(b"<script type='text/javascript' src='/general/scripts.js'></script>")?;
        
        response.send(b"</head>")?;
        
    }
    
    // ---------- body ----------
    
    {
        
        response.send(b"<body>")?;
        
        // ---------- section ----------
        
        {
            
            response.send(b"<div tabindex='0' class='section'>")?;
            
            // ---------- panel ----------
            
            {
                
                response.send(b"<div class='panel'>")?;
                
                // ---------- sections ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<a href='/files/'>files</a>")?;
                    response.send(b"<a href='/watchlist/'>watchlist</a>")?;
                    response.send(b"<a href='/rules/'>rules</a>")?;
                    response.send(b"<a href='/feeds/'>feeds</a>")?;
                    
                    response.send(b"</div>")?;
                    
                }
                
                // ---------- filter ----------
                
                {
                    
                    response.send(b"<input class='filter' placeholder='filter'>")?;
                    
                }
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- list ----------
            
            {
                
                response.send(b"<div tabindex='0' data-refresh='/files/entries' class='list sorted show-containers show-primary'></div>")?;
                
            }
            
            // ---------- panel ----------
            
            {
                
                response.send(b"<div class='panel'>")?;
                
                // ---------- actions ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<a data-hotkey='Enter' onclick='request({ url: \"/files/play\", confirm: false, prompt: false, refresh: false });'>play</a>")?;
                    response.send(b"<a data-hotkey='Delete' onclick='request({ url: \"/files/mark\", confirm: false, prompt: false, refresh: true });'>mark</a>")?;
                    response.send(b"<a data-hotkey='F3' onclick='request({ url: \"/files/move\", confirm: false, prompt: true, refresh: true });'>move</a>")?;
                    response.send(b"<a onclick='request({ url: \"/files/delete\", confirm: true, prompt: false, refresh: true });'>delete</a>")?;
                    
                    response.send(b"</div>")?;
                    
                }
                
                // ---------- toggles ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<label>")?;
                    response.send(b"<input type='checkbox' value='show-containers' checked='checked'>")?;
                    response.send(b"containers")?;
                    response.send(b"</label>")?;
                    
                    response.send(b"<label>")?;
                    response.send(b"<input type='checkbox' value='show-primary' checked='checked'>")?;
                    response.send(b"unwatched")?;
                    response.send(b"</label>")?;
                    
                    response.send(b"<label>")?;
                    response.send(b"<input type='checkbox' value='show-secondary'>")?;
                    response.send(b"watched")?;
                    response.send(b"</label>")?;
                    
                    response.send(b"</div>")?;
                    
                }
                
                response.send(b"</div>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
        response.send(b"</body>")?;
        
    }
    
    response.send(b"</html>")?;
    
    Ok(())
    
}

fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = config.get(b"root")?;
    let flag = config.get(b"flag")?;
    
    // -------------------- list --------------------
    
    let files = ena::Files::new(PathBuf::from(root));
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in files {
        
        if entry.is_marked(flag) {
            response.send(b"<a tabindex='0' class='secondary'>")?;
        } else {
            response.send(b"<a tabindex='0'>")?;
        }
        
        if let Some(container) = entry.container(root) {
            response.send(b"<span>")?;
            response.send(container.as_bytes())?;
            response.send(MAIN_SEPARATOR_STR.as_bytes())?;
            response.send(b"</span>")?;
        }
        
        response.send(entry.name().as_bytes())?;
        
        response.send(b"</a>")?;
        
    }
    
    Ok(())
    
}

fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = config.get(b"root")?;
    let player = config.get(b"player")?;
    
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
    
    Command::new(player).args(files).spawn()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn mark(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = config.get(b"root")?;
    let flag = config.get(b"flag")?;
    
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
        .and_then(|mut response| response.send(b"OK"))
    
}

fn move_to_folder(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = config.get(b"root")?;
    
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
        .and_then(|mut response| response.send(b"OK"))
    
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = config.get(b"root")?;
    
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
        .and_then(|mut response| response.send(b"OK"))
    
}
