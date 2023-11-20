use std::error::Error;

use super::{ Request, StatusCode, ContentType, CacheControl };

pub enum FeedsEndpoint {
    Index,
    Entries,
    Insert,
    Delete,
}

impl FeedsEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/feeds/") => Some(Self::Index),
            (b"GET", b"/feeds/entries") => Some(Self::Entries),
            (b"POST", b"/feeds/insert") => Some(Self::Insert),
            (b"POST", b"/feeds/delete") => Some(Self::Delete),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Entries => entries(&mut request),
            Self::Insert => insert(&mut request),
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
                
                response.send(b"<div tabindex='0' data-refresh='/feeds/entries' class='list show-primary'></div>")?;
                
            }
            
            // ---------- panel ----------
            
            {
                
                response.send(b"<div class='panel'>")?;
                
                // ---------- actions ----------
                
                {
                    
                    response.send(b"<div>")?;
                    
                    response.send(b"<a data-hotkey='Insert' onclick='request({ url: \"/feeds/insert\", confirm: false, prompt: true, refresh: true });'>insert</a>")?;
                    response.send(b"<a data-hotkey='Delete' onclick='request({ url: \"/feeds/delete\", confirm: true, prompt: false, refresh: true });'>delete</a>")?;
                    
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
    
    let feeds = chiaki::List::load("feeds")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in feeds.iter() {
        
        response.send(b"<a tabindex='0'>")?;
        response.send(entry.tag)?;
        response.send(b"</a>")?;
        
    }
    
    Ok(())
    
}

fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("feeds")?;
    
    // -------------------- url --------------------
    
    let url = request.param(b"input")
        .next()
        .ok_or("Url not provided")?;
    
    // -------------------- operation --------------------
    
    list.insert(url, 0)?;
    
    // -------------------- commit --------------------
    
    list.commit()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- list --------------------
    
    let mut list = chiaki::List::load("feeds")?;
    
    // -------------------- url --------------------
    
    let url = request.param(b"tag")
        .next()
        .ok_or("Url not provided")?;
    
    // -------------------- operation --------------------
    
    list.delete(url)?;
    
    // -------------------- commit --------------------
    
    list.commit()?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.send(b"OK"))
    
}
