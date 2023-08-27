mod files;
mod general;
mod comms;

use std::{
    error::Error,
    net::TcpListener,
};

use files::FilesEndpoint;
use general::GeneralEndpoint;
use comms::{ Request, Status, ContentType };

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(rin::Config::load()?.get(b"bind")?)?;
    
    for mut request in listener.incoming().filter_map(Request::get) {
        let resource = request.resource();
        
        if let Some(endpoint) = FilesEndpoint::get(&resource) {
            endpoint.process(request);
            continue;
        }
        
        if let Some(endpoint) = GeneralEndpoint::get(&resource) {
            endpoint.process(request);
            continue;
        }
        
        request.start_response(Status::NotFound, ContentType::Plain)
            .and_then(|mut response| response.send(b"Endpoint not found"))
            .ok();
    }
    
    Ok(())
}
