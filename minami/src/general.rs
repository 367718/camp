use std::{
    error::Error,
    io::Write,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/general.html");
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
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
    }
    
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

fn favicon(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Icon, CacheControl::Static)
        .and_then(|mut response| response.write_all(FAVICON))?;
    
    Ok(())
}

fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Css, CacheControl::Static)
        .and_then(|mut response| response.write_all(STYLES))?;
    
    Ok(())
}

fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Javascript, CacheControl::Static)
        .and_then(|mut response| response.write_all(SCRIPTS))?;
    
    Ok(())
}
