use std::{
    error::Error,
    io::Write,
};

use ayano::{ Request, StatusCode, ContentType, CacheControl };

const INDEX: &[u8] = include_bytes!("../rsc/feeds.html");

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("feeds", max_list_size)?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Dynamic)?;
    
    for entry in list.iter() {
        
        response.write_all(b"<a>")?;
        chikuwa::escape_html(entry.tag, &mut response)?;
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("feeds", max_list_size)?;
    
    let key = request.form_field(b"key")
        .next()
        .ok_or("Invalid key")?;
    
    // -------------------- operation --------------------
    
    list.set(key, 0)?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("feeds", max_list_size)?;
    
    let matcher = request.form_field(b"matcher")
        .next()
        .ok_or("No relevant entry found")?;
    
    // -------------------- operation --------------------
    
    list.delete(matcher)?;
    
    // -------------------- response --------------------
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
