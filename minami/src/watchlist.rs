use std::error::Error;

use super::{ Request, Status, ContentType };

const INDEX_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/watchlist/");
const SCRIPTS_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/watchlist/scripts.js");
const STYLES_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/watchlist/styles.css");

const SCRIPTS: &[u8] = include_bytes!("../rsc/watchlist/scripts.js");
const STYLES: &[u8] = include_bytes!("../rsc/watchlist/styles.css");

pub enum WatchlistEndpoint {
    Index,
    Scripts,
    Styles,
}

impl WatchlistEndpoint {
    
    pub fn get(data: &(&[u8], &[u8])) -> Option<Self> {
        match data {
            INDEX_ENDPOINT => Some(Self::Index),
            SCRIPTS_ENDPOINT => Some(Self::Scripts),
            STYLES_ENDPOINT => Some(Self::Styles),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
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
    
    // -------------------- series --------------------
    
    let database = chiaki::Database::load("series")?;
    
    let mut series: Vec<chiaki::DatabaseEntry> = database.entries().collect();
    
    series.sort_unstable_by_key(|entry| entry.tag);
    
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
        response.send(b"<link rel='stylesheet' type='text/css' href='/watchlist/styles.css'>")?;
        response.send(b"<script type='text/javascript' src='/watchlist/scripts.js'></script>")?;
        
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
                response.send(b"<a href='/preferences/'>preferences</a>")?;
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- filter ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<input class='filter' type='text' placeholder='filter'>")?;
                
                response.send(b"</div>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
        // ---------- list ----------
        
        {
            
            response.send(b"<div class='list show-watched'>")?;
            
            for entry in series {
                
                if entry.value == 0 {
                    response.send(b"<a tabindex='0' class='unwatched'>")?;
                } else {
                    response.send(b"<a tabindex='0' class='watched'>")?;
                }
                
                response.send(b"<div>")?;
                
                response.send(b"<span>")?;
                response.send(entry.tag.as_bytes())?;
                response.send(b"</span>")?;
                
                response.send(b"<span>")?;
                response.send(entry.value.to_string().as_bytes())?;
                response.send(b"</span>")?;
                
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
                
                response.send(b"<a tabindex='0'>add</a>")?;
                response.send(b"<a tabindex='0'>edit</a>")?;
                response.send(b"<a tabindex='0'>remove</a>")?;
                response.send(b"<a tabindex='0'>lookup</a>")?;
                response.send(b"<a tabindex='0'>backup</a>")?;
                
                response.send(b"</div>")?;
                
            }
            
            // ---------- toggles ----------
            
            {
                
                response.send(b"<div>")?;
                
                response.send(b"<label>")?;
                response.send(b"<input type='checkbox' value='show-watched' checked='checked'>")?;
                response.send(b"watched")?;
                response.send(b"</label>")?;
                
                response.send(b"<label>")?;
                response.send(b"<input type='checkbox' value='show-unwatched'>")?;
                response.send(b"unwatched")?;
                response.send(b"</label>")?;
                
                response.send(b"</div>")?;
                
            }
            
            response.send(b"</div>")?;
            
        }
        
    }
    
    response.send(b"</html>")?;
    
    Ok(())
    
}

fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Javascript)
        .and_then(|mut response| response.send(SCRIPTS))
}

fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Css)
        .and_then(|mut response| response.send(STYLES))
}
