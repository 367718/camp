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
    let address = rin::get::<&str>(b"address")?;
    
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
    let endpoint = request.endpoint()
        .ok_or("Invalid request")?;
    
    match endpoint {
        
        // -------------------- general --------------------
        
        b"GET /" => general::index(request),
        b"GET /styles.css" => general::styles(request),
        b"GET /scripts.js" => general::scripts(request),
        
        // -------------------- files --------------------
        
        b"GET /files" => files::index(request),
        b"GET /files/entries" => files::entries(request),
        b"POST /files/play" => files::play(request),
        b"POST /files/mark" => files::mark(request),
        b"POST /files/folder" => files::folder(request),
        b"POST /files/delete" => files::delete(request),
        
        // -------------------- watchlist --------------------
        
        b"GET /watchlist" => watchlist::index(request),
        b"GET /watchlist/entries" => watchlist::entries(request),
        b"POST /watchlist/insert" => watchlist::insert(request),
        b"POST /watchlist/update" => watchlist::update(request),
        b"POST /watchlist/delete" => watchlist::delete(request),
        
        // -------------------- rules --------------------
        
        b"GET /rules" => rules::index(request),
        b"GET /rules/entries" => rules::entries(request),
        b"POST /rules/insert" => rules::insert(request),
        b"POST /rules/update" => rules::update(request),
        b"POST /rules/delete" => rules::delete(request),
        
        // -------------------- feeds --------------------
        
        b"GET /feeds" => feeds::index(request),
        b"GET /feeds/entries" => feeds::entries(request),
        b"POST /feeds/insert" => feeds::insert(request),
        b"POST /feeds/delete" => feeds::delete(request),
        
        // -------------------- mobile --------------------
        
        b"GET /mobile" => mobile::index(request),
        b"GET /mobile/styles.css" => mobile::styles(request),
        
        // -------------------- not found --------------------
        
        _ => Err("Endpoint not found".into()),
        
    }
}
