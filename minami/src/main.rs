#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod general;
mod files;
mod watchlist;
mod rules;
mod feeds;

use std::{
    error::Error,
    io::Write,
    process,
};

use ayano::{
    Server, Request,
    StatusCode, ContentType, CacheControl,
};

fn main() {
    if let Err(error) = process() {
        eprintln!("ERROR: {error}");
        process::exit(1);
    }
}

fn process() -> Result<(), Box<dyn Error>> {
    let address = rin::get::<&str>(b"address")?;
    let server = Server::bind(address)?;
    
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
    let (verb, path) = request.verb()
        .zip(request.path())
        .ok_or("Invalid request")?;
    
    match (verb, path.as_str()) {
        
        // -------------------- general --------------------
        
        ("GET", "/styles.css") => general::styles(request),
        ("GET", "/scripts.js") => general::scripts(request),
        
        // -------------------- files --------------------
        
        ("GET", "/files") => files::index(request),
        ("GET", "/files/entries") => files::entries(request),
        ("POST", "/files/play") => files::play(request),
        ("POST", "/files/mark") => files::mark(request),
        ("POST", "/files/folder") => files::folder(request),
        ("POST", "/files/delete") => files::delete(request),
        
        // -------------------- watchlist --------------------
        
        ("GET", "/watchlist") => watchlist::index(request),
        ("GET", "/watchlist/entries") => watchlist::entries(request),
        ("POST", "/watchlist/insert") => watchlist::insert(request),
        ("POST", "/watchlist/update") => watchlist::update(request),
        ("POST", "/watchlist/delete") => watchlist::delete(request),
        
        // -------------------- rules --------------------
        
        ("GET", "/rules") => rules::index(request),
        ("GET", "/rules/entries") => rules::entries(request),
        ("POST", "/rules/insert") => rules::insert(request),
        ("POST", "/rules/update") => rules::update(request),
        ("POST", "/rules/delete") => rules::delete(request),
        
        // -------------------- feeds --------------------
        
        ("GET", "/feeds") => feeds::index(request),
        ("GET", "/feeds/entries") => feeds::entries(request),
        ("POST", "/feeds/insert") => feeds::insert(request),
        ("POST", "/feeds/delete") => feeds::delete(request),
        
        // -------------------- not found --------------------
        
        _ => Err("Endpoint not found".into()),
        
    }
}
