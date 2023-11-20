use std::{
    error::Error,
    str,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

pub enum RulesEndpoint {
    Index,
    Entries,
    Insert,
    Update,
    Delete,
}

impl RulesEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/rules/") => Some(Self::Index),
            (b"GET", b"/rules/entries") => Some(Self::Entries),
            (b"POST", b"/rules/insert") => Some(Self::Insert),
            (b"POST", b"/rules/update") => Some(Self::Update),
            (b"POST", b"/rules/delete") => Some(Self::Delete),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Entries => entries(&mut request),
            Self::Insert => insert(&mut request),
            Self::Update => update(&mut request),
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
                
                response.send(b"<div tabindex='0' data-refresh='/rules/entries' class='list sorted show-primary'></div>")?;
                
            }
            
            // ---------- panel ----------
            
            {
                
                response.send(b"<div class='panel'>")?;
                
                // ---------- actions ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<a data-hotkey='Insert' onclick='request({ url: \"/rules/insert\", confirm: false, prompt: true, refresh: true });'>insert</a>")?;
                    response.send(b"<a data-hotkey='F2' onclick='request({ url: \"/rules/update\", confirm: false, prompt: true, refresh: true });'>update</a>")?;
                    response.send(b"<a data-hotkey='Delete' onclick='request({ url: \"/rules/delete\", confirm: true, prompt: false, refresh: true });'>delete</a>")?;
                    
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
    
    // -------------------- list --------------------
    
    let rules = chiaki::List::load("rules")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in rules.iter() {
        
        response.send(format!("<a tabindex='0' data-value='{}'>", entry.value).as_bytes())?;
        response.send(entry.tag)?;
        response.send(b"</a>")?;
        
    }
    
    Ok(())
    
}

fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("rules")?;
    
    // -------------------- matcher --------------------
    
    let matcher = request.param(b"input")
        .next()
        .ok_or("Matcher not provided")?;
    
    // -------------------- operation --------------------
    
    list.insert(matcher, 0)?;
    
    // -------------------- commit --------------------
    
    list.commit()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn update(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("rules")?;
    
    // -------------------- matcher and progress --------------------
    
    let matcher = request.param(b"tag")
        .next()
        .ok_or("Matcher not provided")?;
    
    let progress = request.param(b"input")
        .next()
        .and_then(|progress| str::from_utf8(progress).ok())
        .and_then(|progress| progress.parse().ok())
        .ok_or("Progress not provided")?;
    
    // -------------------- operation --------------------
    
    list.update(matcher, progress)?;
    
    // -------------------- commit --------------------
    
    list.commit()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("rules")?;
    
    // -------------------- matcher --------------------
    
    let matcher = request.param(b"tag")
        .next()
        .ok_or("Matcher not provided")?;
    
    // -------------------- operation --------------------
    
    list.delete(matcher)?;
    
    // -------------------- commit --------------------
    
    list.commit()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.send(b"OK"))
    
}
