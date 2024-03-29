use std::{
    error::Error,
    io::Write,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/feeds/index.html");

pub enum FeedsEndpoint {
    Index,
    Entries,
    Insert,
    Delete,
}

impl FeedsEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/feeds")=> Some(Self::Index),
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

fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- list --------------------
    
    let feeds = chiaki::List::load("feeds")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in &feeds {
        
        response.write_all(b"<a>")?;
        
        chikuwa::HtmlEscaper::from(entry.tag)
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- url --------------------
    
    let url = request.param(b"input")
        .next()
        .ok_or("Url not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("feeds")
        .and_then(|mut list| list.insert(url, 0))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}

fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- url --------------------
    
    let url = request.param(b"tag")
        .next()
        .ok_or("Url not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("feeds")
        .and_then(|mut list| list.delete(url))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}
