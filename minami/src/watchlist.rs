use core::fmt::NumBuffer;

use std::{
    error::Error,
    io::Write,
};

use ayano::{ Request, ContentType, CacheControl };

macro_rules! anchor_table {
    ($($value:literal),*) => { [$(concat!("<a data-value='", $value, "'>").as_bytes()),*] };
}

const INDEX: &[u8] = include_bytes!("../rsc/watchlist.html");

const COMMON_VALUES: [&[u8]; 26] = anchor_table![
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
    20, 21, 22, 23, 24, 25
];

pub fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(200, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

pub fn entries(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("watchlist", max_list_size)?;
    
    // -------------------- response --------------------
    
    let mut response = request.start_response(200, ContentType::Html, CacheControl::Dynamic)?;
    
    let mut numbuf = NumBuffer::new();
    
    for entry in list.iter() {
        
        if let Some(&anchor_open) = COMMON_VALUES.get(entry.value as usize) {
            response.write_all(anchor_open)?;
        } else {
            response.write_all(b"<a data-value='")?;
            response.write_all(entry.value.format_into(&mut numbuf).as_bytes())?;
            response.write_all(b"'>")?;
        }
        
        chikuwa::escape_html(entry.tag, &mut response)?;
        response.write_all(b"</a>")?;
        
    }
    
    Ok(())
}

pub fn insert(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("watchlist", max_list_size)?;
    
    let key = request.form_field(b"key")
        .next()
        .ok_or("Invalid key")?;
    
    // TODO: replace with u16::from_ascii in the future (https://github.com/rust-lang/rust/issues/134821)
    let value = request.form_field(b"value")
        .next()
        .as_ref()
        .and_then(|value| str::from_utf8(value).ok())
        .and_then(|value| value.parse().ok())
        .ok_or("Invalid value")?;
    
    // -------------------- operation --------------------
    
    list.set(key, value)?;
    
    // -------------------- response --------------------
    
    request.start_response(200, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn update(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("watchlist", max_list_size)?;
    
    let matcher = request.form_field(b"matcher")
        .next()
        .ok_or("No relevant entry found")?;
    
    // TODO: replace with u16::from_ascii in the future (https://github.com/rust-lang/rust/issues/134821)
    let value = request.form_field(b"value")
        .next()
        .as_ref()
        .and_then(|value| str::from_utf8(value).ok())
        .and_then(|value| value.parse().ok())
        .ok_or("Invalid value")?;
    
    // -------------------- operation --------------------
    
    list.set(matcher, value)?;
    
    // -------------------- response --------------------
    
    request.start_response(200, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}

pub fn delete(request: &mut Request) -> Result<(), Box<dyn Error>> {
    // -------------------- params --------------------
    
    let max_list_size = rin::get::<u64>(b"max_list_size")?;
    
    let list = chiaki::List::load("watchlist", max_list_size)?;
    
    let matcher = request.form_field(b"matcher")
        .next()
        .ok_or("No relevant entry found")?;
    
    // -------------------- operation --------------------
    
    list.delete(matcher)?;
    
    // -------------------- response --------------------
    
    request.start_response(200, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
