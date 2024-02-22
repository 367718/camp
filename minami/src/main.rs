#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod files;
mod watchlist;
mod rules;
mod feeds;
mod general;

use std::{
    error::Error,
    io::Write,
};

use files::FilesEndpoint;
use watchlist::WatchlistEndpoint;
use rules::RulesEndpoint;
use feeds::FeedsEndpoint;
use general::GeneralEndpoint;

use ayano::{ Server, Request, StatusCode, ContentType, CacheControl };

fn main() -> Result<(), Box<dyn Error>> {
    for mut request in Server::new(rin::get(b"address")?)? {
        
        let Some(resource) = request.resource() else {
            continue;
        };
        
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
