use std::error::Error;

use super::{ Request, Status, ContentType };

pub enum WatchlistEndpoint {
    Index,
    Add,
    Edit,
    Remove,
}

impl WatchlistEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/watchlist/") => Some(Self::Index),
            (b"POST", b"/watchlist/add") => Some(Self::Add),
            (b"POST", b"/watchlist/edit") => Some(Self::Edit),
            (b"POST", b"/watchlist/remove") => Some(Self::Remove),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Add => add(&mut request),
            Self::Edit => edit(&mut request),
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
    
    let database = chiaki::Database::load("watchlist")?;
    
    let mut watchlist: Vec<chiaki::DatabaseEntry> = database.entries().collect();
    
    watchlist.sort_unstable_by_key(|entry| entry.tag.to_ascii_uppercase());
    
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
                    response.send(b"<span>watchlist</span>")?;
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
                
                response.send(b"<div class='list show-value show-primary'>")?;
                
                for entry in watchlist {
                    
                    response.send(b"<a")?;
                    
                    if entry.value == 0 {
                        response.send(b" class='secondary'")?;
                    }
                    
                    response.send(b" data-value='")?;
                    response.send(entry.value.to_string().as_bytes())?;
                    response.send(b"'>")?;
                    
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
                    
                    response.send(b"<a onclick='request({ url: \"/watchlist/add\", confirm: false, prompt: true, refresh: true });'>add</a>")?;
                    response.send(b"<a onclick='request({ url: \"/watchlist/edit\", confirm: false, prompt: true, refresh: true });'>edit</a>")?;
                    response.send(b"<a onclick='request({ url: \"/watchlist/remove\", confirm: true, prompt: false, refresh: true });'>remove</a>")?;
                    response.send(b"<a onclick='request({ url: \"/general/lookup\", confirm: false, prompt: false, refresh: false });'>lookup</a>")?;
                    
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
                    response.send(b"<input type='checkbox' value='show-secondary'>")?;
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

fn add(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- database --------------------
    
    let database = chiaki::Database::load("watchlist")?;
    
    // -------------------- title --------------------
    
    let title = request.param(b"input")
        .next()
        .ok_or("Title not provided")?;
    
    // -------------------- operation --------------------
    
    database.add(title, 0)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn edit(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- database --------------------
    
    let database = chiaki::Database::load("watchlist")?;
    
    // -------------------- title and progress --------------------
    
    let title = request.param(b"tag")
        .next()
        .ok_or("Title not provided")?;
    
    let progress = request.param(b"input")
        .next()
        .and_then(|progress| progress.parse().ok())
        .ok_or("Progress not provided")?;
    
    // -------------------- operation --------------------
    
    database.edit(title, progress)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}

fn remove(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
    // -------------------- database --------------------
    
    let database = chiaki::Database::load("watchlist")?;
    
    // -------------------- title --------------------
    
    let title = request.param(b"tag")
        .next()
        .ok_or("Title not provided")?;
    
    // -------------------- operation --------------------
    
    database.remove(title)?;
    
    // -------------------- response --------------------
    
    request.start_response(Status::Ok, ContentType::Plain)
        .and_then(|mut response| response.send(b"OK"))
    
}
