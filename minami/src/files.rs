use std::{
    error::Error,
    path::{ MAIN_SEPARATOR_STR, Path, PathBuf },
    process::Command,
};

use super::{ Request, Status, ContentType };

const INDEX_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/files/");
const PLAY_ENDPOINT: &(&[u8], &[u8]) = &(b"POST", b"/files/play");
const MARK_ENDPOINT: &(&[u8], &[u8]) = &(b"POST", b"/files/mark");
const MOVE_ENDPOINT: &(&[u8], &[u8]) = &(b"POST", b"/files/move");
const LOOKUP_ENDPOINT: &(&[u8], &[u8]) = &(b"POST", b"/files/lookup");
const SCRIPTS_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/files/scripts.js");
const STYLES_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/files/styles.css");

const SCRIPTS: &[u8] = include_bytes!("../rsc/files/scripts.js");
const STYLES: &[u8] = include_bytes!("../rsc/files/styles.css");

pub enum FilesEndpoint {
    Index,
    Play,
    Mark,
    Move,
    Lookup,
    Scripts,
    Styles,
}

impl FilesEndpoint {
    
    pub fn get(data: &(&[u8], &[u8])) -> Option<Self> {
        match data {
            INDEX_ENDPOINT => Some(Self::Index),
            PLAY_ENDPOINT => Some(Self::Play),
            MARK_ENDPOINT => Some(Self::Mark),
            MOVE_ENDPOINT => Some(Self::Move),
            LOOKUP_ENDPOINT => Some(Self::Lookup),
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
            Self::Move => move_to_folder(&mut request),
            Self::Lookup => lookup(&mut request),
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
    
    files.sort_unstable_by_key(|entry| (entry.container(root).is_some(), entry.path().to_uppercase()));
    
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
                
                response.send(b"<a href='/watchlist/'>watchlist</a>")?;
                response.send(b"<a href='/rules/'>rules</a>")?;
                response.send(b"<a href='/feeds/'>feeds</a>")?;
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- filter ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<input class='filter' placeholder='filter'>")?;
                
                response.send(b"</div>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
        // ---------- list ----------
        
        {
            
            response.send(b"<div class='list show-containers show-new'>")?;
            
            for entry in files {
                
                if entry.is_marked(flag) {
                    response.send(b"<a tabindex='0' class='watched'>")?;
                } else {
                    response.send(b"<a tabindex='0'>")?;
                }
                
                response.send(b"<div>")?;
                
                if let Some(container) = entry.container(root) {
                    response.send(b"<span>")?;
                    response.send(container.as_bytes())?;
                    response.send(MAIN_SEPARATOR_STR.as_bytes())?;
                    response.send(b"</span>")?;
                }
                
                response.send(entry.name().as_bytes())?;
                
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
                
                response.send(b"<a tabindex='0' onclick='play();'>play</a>")?;
                response.send(b"<a tabindex='0' onclick='mark();'>mark</a>")?;
                response.send(b"<a tabindex='0' onclick='move();'>move</a>")?;
                response.send(b"<a tabindex='0'>delete</a>")?;
                response.send(b"<a tabindex='0' onclick='lookup();'>lookup</a>")?;
                response.send(b"<a tabindex='0'>download</a>")?;
                response.send(b"<a tabindex='0'>control</a>")?;
                
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
        
    }
    
    response.send(b"</html>")?;
    
    Ok(())
    
}

fn play(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = Path::new(config.get(b"root")?).canonicalize().map_err(|_| "Invalid root directory")?;
    let command = config.get(b"command")?;
    
    // -------------------- paths --------------------
    
    let mut paths = request.values(b"path")
        .filter_map(|path| root.join(path).canonicalize().ok())
        .filter(|path| path.starts_with(&root))
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
    let root = Path::new(config.get(b"root")?).canonicalize().map_err(|_| "Invalid root directory")?;
    let flag = config.get(b"flag")?;
    
    // -------------------- files --------------------
    
    let mut files = request.values(b"path")
        .filter_map(|path| root.join(path).canonicalize().ok())
        .filter(|path| path.starts_with(&root))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File path not provided".into());
    }
    
    // -------------------- operation --------------------
    
    for mut entry in files {
        entry.mark(flag, ! entry.is_marked(flag))?;
    }
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn move_to_folder(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let root = Path::new(config.get(b"root")?).canonicalize().map_err(|_| "Invalid root directory")?;
    
    // -------------------- folder --------------------
    
    let folder = Path::new(request.value(b"folder").ok_or("Folder name not provided")?)
        .file_name().ok_or("Invalid folder name")?;
    
    // -------------------- files --------------------
    
    let mut files = request.values(b"path")
        .filter_map(|path| root.join(path).canonicalize().ok())
        .filter(|path| path.starts_with(&root))
        .filter_map(|path| ena::Files::new(path).next())
        .peekable();
    
    if files.peek().is_none() {
        return Err("File path not provided".into());
    }
    
    // -------------------- operation --------------------
    
    for mut entry in files {
        entry.move_to_folder(folder)?;
    }
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn lookup(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- config --------------------
    
    let config = rin::Config::load()?;
    let lookup = config.get(b"lookup")?;
    
    // -------------------- path --------------------
    
    let path = request.value(b"path").ok_or("File path not provided")?;
    
    // -------------------- operation --------------------
    
    chikuwa::execute_app(&lookup.replace("%s", &chikuwa::percent_encode(path)))?;
    
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
