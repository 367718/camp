use std::error::Error;

use super::{ Request, Status, ContentType };

const FAVICON: &[u8] = include_bytes!("../rsc/favicon.ico");
const STYLES: &[u8] = include_bytes!("../rsc/styles.css");
const SCRIPTS: &[u8] = include_bytes!("../rsc/scripts.js");

pub enum GeneralEndpoint {
    Index,
    Favicon,
    Styles,
    Scripts,
}

impl GeneralEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/") => Some(Self::Index),
            (b"GET", b"/general/favicon.ico") => Some(Self::Favicon),
            (b"GET", b"/general/styles.css") => Some(Self::Styles),
            (b"GET", b"/general/scripts.js") => Some(Self::Scripts),
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Index => index(&mut request),
            Self::Favicon => favicon(&mut request),
            Self::Styles => styles(&mut request),
            Self::Scripts => scripts(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(Status::Error, ContentType::Plain)
                .and_then(|mut response| response.send(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
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
        
        response.send(b"</head>")?;
        
    }
    
    // ---------- body ----------
    
    {
        
        response.send(b"<body>")?;
        
        response.send(b"<iframe src='/files/'></iframe>")?;
        response.send(b"<iframe src='/watchlist/'></iframe>")?;
        
        response.send(b"</body>")?;
        
    }
    
    response.send(b"</html>")?;
    
    Ok(())
    
}

fn favicon(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Favicon)
        .and_then(|mut response| response.send(FAVICON))
}

fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Styles)
        .and_then(|mut response| response.send(STYLES))
}

fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(Status::Ok, ContentType::Scripts)
        .and_then(|mut response| response.send(SCRIPTS))
}
