use std::{
    error::Error,
    str,
};

use super::{ Request, Status, ContentType };

pub enum WatchlistEndpoint {
    Index,
    Insert,
    Update,
    Delete,
}

impl WatchlistEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/watchlist/") => Some(Self::Index),
            (b"POST", b"/watchlist/insert") => Some(Self::Insert),
            (b"POST", b"/watchlist/update") => Some(Self::Update),
            (b"POST", b"/watchlist/delete") => Some(Self::Delete),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Insert => insert(&mut request),
            Self::Update => update(&mut request),
            Self::Delete => delete(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(Status::Error, ContentType::Plain)
                .and_then(|mut response| response.send(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let watchlist = chiaki::List::load("watchlist")?;
    
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
        response.send(b"<link rel='stylesheet' type='text/css' href='/general/styles.css'>")?;
        response.send(b"<script type='text/javascript' src='/general/scripts.js'></script>")?;
        
        response.send(b"</head>")?;
        
    }
    
    // ---------- body ----------
    
    {
        
        response.send(b"<body>")?;
        
        // ---------- section ----------
        
        {
            
            response.send(b"<div class='section'>")?;
            
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
                
                response.send(b"<div class='list sorted show-value show-primary show-secondary'>")?;
                
                for entry in watchlist.iter() {
                    
                    response.send(b"<a")?;
                    
                    if entry.value == 0 {
                        response.send(b" class='secondary'")?;
                    }
                    
                    response.send(b" data-value='")?;
                    response.send(format!("{}", entry.value).as_bytes())?;
                    response.send(b"'>")?;
                    
                    response.send(entry.tag)?;
                    
                    response.send(b"</a>")?;
                    
                }
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- panel ----------
            
            {
                
                response.send(b"<div class='panel'>")?;
                
                // ---------- actions ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<a data-hotkey='Insert' onclick='request({ url: \"/watchlist/insert\", confirm: false, prompt: true, refresh: true });'>insert</a>")?;
                    response.send(b"<a data-hotkey='F2' onclick='request({ url: \"/watchlist/update\", confirm: false, prompt: true, refresh: true });'>update</a>")?;
                    response.send(b"<a data-hotkey='Delete' onclick='request({ url: \"/watchlist/delete\", confirm: true, prompt: false, refresh: true });'>delete</a>")?;
                    
                    response.send(b"</div>")?;
                    
                }
                
                // ---------- toggles ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<label>")?;
                    response.send(b"<input type='checkbox' value='show-primary' checked='checked'>")?;
                    response.send(b"watched")?;
                    response.send(b"</label>")?;
                    
                    response.send(b"<label>")?;
                    response.send(b"<input type='checkbox' value='show-secondary' checked='checked'>")?;
                    response.send(b"unwatched")?;
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

fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("watchlist")?;
    
    // -------------------- title --------------------
    
    let title = request.param(b"input")
        .next()
        .ok_or("Title not provided")?;
    
    // -------------------- operation --------------------
    
    list.insert(title, 0)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn update(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("watchlist")?;
    
    // -------------------- title and progress --------------------
    
    let title = request.param(b"tag")
        .next()
        .ok_or("Title not provided")?;
    
    let progress = request.param(b"input")
        .next()
        .and_then(|progress| str::from_utf8(progress).ok())
        .and_then(|progress| progress.parse().ok())
        .ok_or("Progress not provided")?;
    
    // -------------------- operation --------------------
    
    list.update(title, progress)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("watchlist")?;
    
    // -------------------- title --------------------
    
    let title = request.param(b"tag")
        .next()
        .ok_or("Title not provided")?;
    
    // -------------------- operation --------------------
    
    list.delete(title)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}
