use std::{
    error::Error,
    io::Write,
    str,
};

use ayano::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/watchlist/index.html");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- list --------------------
    
    let watchlist = chiaki::List::load("watchlist")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in &watchlist {
        
        write!(&mut response, "<a data-value='{}'>", entry.value)?;
        
        chikuwa::HtmlEscaper::from(entry.tag)
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- title --------------------
    
    let title = request.param(b"input")
        .next()
        .ok_or("Title not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("watchlist")
        .and_then(|mut list| list.insert(title, 0))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn update(request: &mut Request) -> Result<(), Box<dyn Error>> {
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
    
    chiaki::List::load("watchlist")
        .and_then(|mut list| list.update(title, progress))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- title --------------------
    
    let title = request.param(b"tag")
        .next()
        .ok_or("Title not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("watchlist")
        .and_then(|mut list| list.delete(title))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
