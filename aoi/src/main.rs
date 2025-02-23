use std::{
    error::Error,
    fs::OpenOptions,
    io::{ self, Read, Write },
    os::raw::*,
};

use ayano::{
    Server, Request,
    StatusCode, ContentType, CacheControl,
};

unsafe extern "system" {
    
    // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew
    fn WaitNamedPipeW(
        lpNamedPipeName: *const c_ushort,
        nTimeOut: c_ulong,
    ) -> c_int;
    
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const INDEX: &[u8] = include_bytes!("../rsc/index.html");

const PIPE_MAX_WAIT: c_ulong = 5000; // milliseconds

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
            
            write_to_named_pipe(rin::get(b"pipe")?, command)?;
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

fn write_to_named_pipe(path: &str, data: &[u8]) -> io::Result<()> {
    unsafe {
            
        let result = WaitNamedPipeW(
            chikuwa::win_string(path).as_ptr(),
            PIPE_MAX_WAIT,
        );
        
        if result == 0 {
            return Err(io::Error::last_os_error());
        }
        
    }
    
    OpenOptions::new()
        .write(true)
        .open(path)
        .and_then(|mut pipe| pipe.write_all(data))
}
