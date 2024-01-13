#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod files;
mod watchlist;
mod rules;
mod feeds;
mod general;
mod networking;

use std::{
    error::Error,
    io::Write,
    net::TcpListener,
};

use files::FilesEndpoint;
use watchlist::WatchlistEndpoint;
use rules::RulesEndpoint;
use feeds::FeedsEndpoint;
use general::GeneralEndpoint;
use networking::{ Request, StatusCode, ContentType, CacheControl };

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(rin::get(b"address")?)?;
    
    for mut request in listener.incoming().filter_map(Request::get) {
        let resource = request.resource();
        
        if let Some(endpoint) = FilesEndpoint::get(resource) {
            endpoint.process(request);
            continue;
        }
        
        if let Some(endpoint) = WatchlistEndpoint::get(resource) {
            endpoint.process(request);
            continue;
        }
        
        if let Some(endpoint) = RulesEndpoint::get(resource) {
            endpoint.process(request);
            continue;
        }
        
        if let Some(endpoint) = FeedsEndpoint::get(resource) {
            endpoint.process(request);
            continue;
        }
        
        if let Some(endpoint) = GeneralEndpoint::get(resource) {
            endpoint.process(request);
            continue;
        }
        
        request.start_response(StatusCode::NotFound, ContentType::Plain, CacheControl::Dynamic)
            .and_then(|mut response| response.write_all(b"Endpoint not found"))
            .ok();
    }
    
    Ok(())
}
