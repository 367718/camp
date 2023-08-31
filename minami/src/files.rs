use std::{
    error::Error,
    path::{ MAIN_SEPARATOR_STR, PathBuf },
    process::Command,
};

use super::{ Request, Status, ContentType };

const INDEX_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/files/");
const PLAY_ENDPOINT: &(&[u8], &[u8]) = &(b"POST", b"/files/play");
const MARK_ENDPOINT: &(&[u8], &[u8]) = &(b"POST", b"/files/mark");
const SCRIPTS_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/files/scripts.js");
const STYLES_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/files/styles.css");

const SCRIPTS: &[u8] = include_bytes!("../rsc/files/scripts.js");
const STYLES: &[u8] = include_bytes!("../rsc/files/styles.css");

pub enum FilesEndpoint {
    Index,
    Play,
    Mark,
    Scripts,
    Styles,
}

impl FilesEndpoint {
    
    pub fn get(data: &(&[u8], &[u8])) -> Option<Self> {
        match data {
            INDEX_ENDPOINT => Some(Self::Index),
            PLAY_ENDPOINT => Some(Self::Play),
            MARK_ENDPOINT => Some(Self::Mark),
            SCRIPTS_ENDPOINT => Some(Self::Scripts),
            STYLES_ENDPOINT => Some(Self::Styles),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Play => play(&mut request),
            Self::Mark => mark(&mut request),
            Self::Scripts => scripts(&mut request),
            Self::Styles => styles(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(Status::Error, ContentType::Plain)
                .and_then(|mut response| response.send(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = config.get(b"root")?;
    let flag = config.get(b"flag")?;
    
    // -------------------- files --------------------
    
    let mut files: Vec<ena::FilesEntry> = ena::Files::new(PathBuf::from(root)).collect();
    
    files.sort_unstable_by_key(|file| (file.container(root).is_some(), file.path().to_uppercase()));
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(Status::Ok, ContentType::Html)?;
    
    response.send(b"<!DOCTYPE html>")?;
    response.send(b"<html lang='en'>")?;
    
    // ---------- head ----------
    
    {
        
        response.send(b"<head>")?;
        
        response.send(b"<meta charset='utf-8'>")?;
        response.send(b"<meta name='viewport' content='width=device-width, initial-scale=1'>")?;
        response.send(b"<title>minami</title>")?;
        response.send(b"<link rel='icon' type='image/x-icon' href='/general/favicon.ico'>")?;
        response.send(b"<link rel='stylesheet' type='text/css' href='/files/styles.css'>")?;
        response.send(b"<script type='text/javascript' src='/files/scripts.js'></script>")?;
        
        response.send(b"</head>")?;
        
    }
    
    // ---------- body ----------
    
    {
        
        response.send(b"<body>")?;
        
        // ---------- panel ----------
        
        {
            
            response.send(b"<div class='panel'>")?;
            
            // ---------- sections ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<a href='/files/'>files</a>")?;
                response.send(b"<a href='/watchlist/'>watchlist</a>")?;
                response.send(b"<a href='/tools/'>tools</a>")?;
                response.send(b"<a href='/preferences/'>preferences</a>")?;
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- filters ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<label>")?;
                response.send(b"<input type='checkbox' value='show-containers' checked='checked'>")?;
                response.send(b"containers")?;
                response.send(b"</label>")?;
                
                response.send(b"<label>")?;
                response.send(b"<input type='checkbox' value='show-new' checked='checked'>")?;
                response.send(b"new")?;
                response.send(b"</label>")?;
                
                response.send(b"<label>")?;
                response.send(b"<input type='checkbox' value='show-watched'>")?;
                response.send(b"watched")?;
                response.send(b"</label>")?;
                
                response.send(b"</div>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
        // ---------- list ----------
        
        {
            
            response.send(b"<div class='list show-containers show-new'>")?;
            
            for file in files {
                
                response.send(b"<a tabindex='0' class='")?;
                response.send(file.mark(flag).as_class().as_bytes())?;
                response.send(b"'>")?;
                
                response.send(b"<div>")?;
                
                if let Some(container) = file.container(root) {
                    response.send(b"<span>")?;
                    response.send(container.as_bytes())?;
                    response.send(MAIN_SEPARATOR_STR.as_bytes())?;
                    response.send(b"</span>")?;
                }
                
                response.send(file.name().as_bytes())?;
                
                response.send(b"</div>")?;
                
                response.send(b"</a>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
        response.send(b"</body>")?;
        
        // ---------- panel ----------
        
        {
            
            response.send(b"<div class='panel'>")?;
            
            // ---------- actions ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<a>play</a>")?;
                response.send(b"<a>mark</a>")?;
                response.send(b"<a>rename</a>")?;
                response.send(b"<a>move</a>")?;
                response.send(b"<a>delete</a>")?;
                response.send(b"<a>lookup</a>")?;
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- tools ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<a>download new releases</a>")?;
                
                response.send(b"</div>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
    }
    
    response.send(b"</html>")?;
    
    Ok(())
    
}

fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = PathBuf::from(config.get(b"root")?);
    let command = config.get(b"command")?;
    
    // -------------------- paths --------------------
    
    let mut paths = request.values(b"path")
        .map(|path| root.join(path))
        .peekable();
    
    if paths.peek().is_none() {
        return Err("File path not provided".into());
    }
    
    // -------------------- operation --------------------
    
    Command::new(command).args(paths).spawn()?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn mark(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = PathBuf::from(config.get(b"root")?);
    let flag = config.get(b"flag")?;
    
    // -------------------- files --------------------
    
    let mut files = request.values(b"path")
        .map(|path| root.join(path))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File path not provided".into());
    }
    
    // -------------------- operation --------------------
    
    for mut file in files {
        
        let mark = match file.mark(flag) {
            ena::FilesMark::None => ena::FilesMark::Watched,
            ena::FilesMark::Watched => ena::FilesMark::None,
            ena::FilesMark::Updated => continue,
        };
        
        file.set_mark(flag, mark)?;
        
    }
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Javascript)
        .and_then(|mut response| response.send(SCRIPTS))
}

fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Css)
        .and_then(|mut response| response.send(STYLES))
}
