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
    // -------------------- operation --------------------
    
    let list = chiaki::List::load("watchlist")?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in &list {
        
        write!(&mut response, "<a data-value='{}'>", entry.value)?;
        
        chikuwa::HtmlEscaper::from(entry.tag)
            .try_for_each(|escaped| response.write_all(escaped))?;
        
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let input = request.param(b"input")
        .next()
        .ok_or("Input not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("watchlist")
        .and_then(|mut list| list.insert(input, 0))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn update(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let matcher = request.param(b"matcher")
        .next()
        .ok_or("Matcher not provided")?;
    
    let input = request.param(b"input")
        .next()
        .and_then(|input| str::from_utf8(input).ok())
        .and_then(|input| input.parse().ok())
        .ok_or("Input not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("watchlist")
        .and_then(|mut list| list.update(matcher, input))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let matcher = request.param(b"matcher")
        .next()
        .ok_or("Matcher not provided")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("watchlist")
        .and_then(|mut list| list.delete(matcher))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
