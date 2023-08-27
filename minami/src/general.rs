use std::error::Error;

use super::{ Request, Status, ContentType };

const ROOT_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/");
const STYLES_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/general/styles.css");
const FAVICON_ENDPOINT: &(&[u8], &[u8]) = &(b"GET", b"/general/favicon.ico");

const FAVICON: &[u8] = include_bytes!("../rsc/general/favicon.ico");
const STYLES: &[u8] = include_bytes!("../rsc/general/styles.css");

pub enum GeneralEndpoint {
    Root,
    Favicon,
    Styles,    
}

impl GeneralEndpoint {
    
    pub fn get(data: &(&[u8], &[u8])) -> Option<Self> {
        match data {
            ROOT_ENDPOINT => Some(Self::Root),
            FAVICON_ENDPOINT => Some(Self::Favicon),
            STYLES_ENDPOINT => Some(Self::Styles),            
            _ => None,
        }
    }
    
    pub fn process(&self, mut request: Request) {
        let result = match self {
            Self::Root => root(&mut request),
            Self::Favicon => favicon(&mut request),
            Self::Styles => styles(&mut request),
        };
        
        if let Err(error) = result {
            request.start_response(Status::Error, ContentType::Plain)
                .and_then(|mut response| response.send(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn root(request: &mut Request) -> Result<(), Box<dyn Error>> {
    
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
        
        response.send(b"<iframe src='/files/' title='files'></iframe>")?;
        response.send(b"<iframe src='/files/' title='files'></iframe>")?;
        
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
    request.start_response(Status::Ok, ContentType::Css)
        .and_then(|mut response| response.send(STYLES))
}
