use std::{
    error::Error,
    io::Write,
    str,
};

use super::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/watchlist/index.html");

pub enum WatchlistEndpoint {
    Index,
    Entries,
    Insert,
    Update,
    Delete,
}

impl WatchlistEndpoint {
    
    pub fn get(resource: (&[u8], &[u8])) -> Option<Self> {
        match resource {
            (b"GET", b"/watchlist") => Some(Self::Index),
            (b"GET", b"/watchlist/entries") => Some(Self::Entries),
            (b"POST", b"/watchlist/insert") => Some(Self::Insert),
            (b"POST", b"/watchlist/update") => Some(Self::Update),
            (b"POST", b"/watchlist/delete") => Some(Self::Delete),
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
    
    let watchlist = chiaki::List::load("watchlist")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in watchlist.iter() {
        
        write!(&mut response, "<a data-value='{}'>", entry.value)?;
        
        chikuwa::HtmlEscaper::from(entry.tag)
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
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
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
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
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
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
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
        .and_then(|mut response| response.write_all(b"OK"))?;
    
    Ok(())
}
