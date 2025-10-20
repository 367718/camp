#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod general;
mod files;
mod watchlist;
mod rules;
mod feeds;
mod mobile;

use std::{
    error::Error,
    io::Write,
};

use ayano::{
    Server, Request,
    StatusCode, ContentType, CacheControl,
};

fn main() -> Result<(), Box<dyn Error>> {
    let address = rin::get(b"address")?;
    let mut server = Server::bind(address)?;
    
    loop {
        
        let Ok(mut request) = server.accept() else {
            continue;
        };
        
        if let Err(error) = handle_request(&mut request) {
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
        
    }
}

fn handle_request(request: &mut Request) -> Result<(), Box<dyn Error>> {
    let (method, path) = request.method_and_path()
        .ok_or("Invalid request")?;
    
    match (method, path) {
        
        // -------------------- general --------------------
        
        (b"GET", b"/") => general::index(request),
        (b"GET", b"/styles.css") => general::styles(request),
        (b"GET", b"/scripts.js") => general::scripts(request),
        
        // -------------------- files --------------------
        
        (b"GET", b"/files") => files::index(request),
        (b"GET", b"/files/entries") => files::entries(request),
        (b"POST", b"/files/play") => files::play(request),
        (b"POST", b"/files/mark") => files::mark(request),
        (b"POST", b"/files/folder") => files::folder(request),
        (b"POST", b"/files/delete") => files::delete(request),
        
        // -------------------- watchlist --------------------
        
        (b"GET", b"/watchlist") => watchlist::index(request),
        (b"GET", b"/watchlist/entries") => watchlist::entries(request),
        (b"POST", b"/watchlist/insert") => watchlist::insert(request),
        (b"POST", b"/watchlist/update") => watchlist::update(request),
        (b"POST", b"/watchlist/delete") => watchlist::delete(request),
        
        // -------------------- rules --------------------
        
        (b"GET", b"/rules") => rules::index(request),
        (b"GET", b"/rules/entries") => rules::entries(request),
        (b"POST", b"/rules/insert") => rules::insert(request),
        (b"POST", b"/rules/update") => rules::update(request),
        (b"POST", b"/rules/delete") => rules::delete(request),
        
        // -------------------- feeds --------------------
        
        (b"GET", b"/feeds") => feeds::index(request),
        (b"GET", b"/feeds/entries") => feeds::entries(request),
        (b"POST", b"/feeds/insert") => feeds::insert(request),
        (b"POST", b"/feeds/delete") => feeds::delete(request),
        
        // -------------------- mobile --------------------
        
        (b"GET", b"/mobile") => mobile::index(request),
        (b"GET", b"/mobile/styles.css") => mobile::styles(request),
        
        // -------------------- not found --------------------
        
        _ => Err("Endpoint not found".into()),
        
    }
}
