use std::error::Error;

use super::{ Request, Status, ContentType };

pub enum FeedsEndpoint {
    Index,
    Add,
    Remove,
}

impl FeedsEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/feeds/") => Some(Self::Index),
            (b"POST", b"/feeds/add") => Some(Self::Add),
            (b"POST", b"/feeds/remove") => Some(Self::Remove),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Add => add(&mut request),
            Self::Remove => remove(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(Status::Error, ContentType::Plain)
                .and_then(|mut response| response.send(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- database --------------------
    
    let database = chiaki::Database::load("feeds")?;
    
    let mut feeds: Vec<chiaki::DatabaseEntry> = database.entries().collect();
    
    feeds.sort_unstable_by_key(|entry| entry.tag.to_ascii_uppercase());
    
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
                    response.send(b"<span>feeds</span>")?;
                    
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
                
                response.send(b"<div class='list'>")?;
                
                for entry in feeds {
                    
                    response.send(b"<a>")?;
                    response.send(entry.tag.as_bytes())?;
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
                    
                    response.send(b"<a data-hotkey='Insert' onclick='request({ url: \"/feeds/add\", confirm: false, prompt: true, refresh: true });'>add</a>")?;
                    response.send(b"<a data-hotkey='Delete' onclick='request({ url: \"/feeds/remove\", confirm: true, prompt: false, refresh: true });'>remove</a>")?;
                    
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

fn add(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- database --------------------
    
    let database = chiaki::Database::load("feeds")?;
    
    // -------------------- url --------------------
    
    let url = request.param(b"input")
        .next()
        .ok_or("Url not provided")?;
    
    // -------------------- operation --------------------
    
    database.add(url, 0)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn remove(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- database --------------------
    
    let database = chiaki::Database::load("feeds")?;
    
    // -------------------- url --------------------
    
    let url = request.param(b"tag")
        .next()
        .ok_or("Url not provided")?;
    
    // -------------------- operation --------------------
    
    database.remove(url)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}
