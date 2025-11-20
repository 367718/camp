use std::{
    error::Error,
    io::Write,
};

use ayano::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/feeds/index.html");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let filter = request.query_string(b"filter")
        .next()
        .unwrap_or_default();
    
    // -------------------- operation --------------------
    
    let list = chiaki::List::load("feeds", max_list_size)?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in &list {
        
        if ! filter.is_empty() && ! chikuwa::insensitive_contains(entry.tag, &filter) {
            continue;
        }
        
        response.write_all(b"<a>")?;
        chikuwa::escape_html(&mut response, entry.tag)?;
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let input = request.form_params(b"input")
        .next()
        .ok_or("Wrong input")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("feeds", max_list_size)
        .and_then(|list| list.set(&input, 0))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let matcher = request.form_params(b"matcher")
        .next()
        .ok_or("Wrong matcher")?;
    
    // -------------------- operation --------------------
    
    chiaki::List::load("feeds", max_list_size)
        .and_then(|list| list.delete(&matcher))?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
