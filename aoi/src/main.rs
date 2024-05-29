use std::{
    error::Error,
    io::{ self, Read, Write },
};

use ayano::{ Server, Request, StatusCode, ContentType, CacheControl };

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const INDEX: &[u8] = include_bytes!("../rsc/index.html");

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);
    
    if let Err(error) = process() {
        println!();
        println!("ERROR: {}", error);
    }
    
    println!();
    print!("Press 'enter' key to exit...");
    
    io::stdout().flush().unwrap();
    let _ = io::stdin().read(&mut [0]).unwrap();
}

fn process() -> Result<(), Box<dyn Error>> {
    let address = rin::get(b"address")?;
    let server = Server::bind(address)?;
    
    println!();
    println!("Listening on {}", address);
    
    for mut request in server.flatten() {
        
        if let Err(error) = handle_request(&mut request) {
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
        
    }
    
    Ok(())
}

fn handle_request(request: &mut Request) -> Result<(), Box<dyn Error>> {
    let (method, path) = request.resource()
        .ok_or("Invalid request")?;
    
    if method == b"GET" {
        
        // -------------------- index --------------------
        
        if path == b"/" {
            
            request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
                .and_then(|mut response| response.write_all(INDEX))?;
            
            return Ok(());
            
        }
        
        // -------------------- commands --------------------
        
        if let Some(command) = get_command(path) {
            
            chikuwa::NamedPipe::connect(rin::get(b"name")?)?
                .write_all(command)?;
            
            request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
            
            return Ok(());
            
        }
        
    }
    
    // -------------------- not found --------------------
    
    Err("Endpoint not found".into())
}

fn get_command(path: &[u8]) -> Option<&'static [u8]> {
    match path {
        b"/play" => Some(b"cycle pause\n"),
        b"/minuschapter" => Some(b"cycle chapter down\n"),
        b"/pluschapter" => Some(b"cycle chapter up\n"),
        b"/minusplaylist" => Some(b"playlist-prev\n"),
        b"/plusplaylist" => Some(b"playlist-next\n"),
        b"/minus5" => Some(b"seek -5\n"),
        b"/plus5" => Some(b"seek 5\n"),
        b"/minus75" => Some(b"seek -75\n"),
        b"/plus75" => Some(b"seek 75\n"),
        b"/fullscreen" => Some(b"cycle fullscreen\n"),
        b"/subtitles" => Some(b"cycle sub\n"),
        b"/title" => Some(b"show-text ${media-title} 5000\n"),
        b"/time" => Some(b"show-text \"${playback-time} (${time-remaining})\" 5000\n"),
        b"/quit" => Some(b"quit\n"),
        _ => None,
    }
}
