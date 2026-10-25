#[link(name = "kernel32")]
unsafe extern "system" {}

mod sys;

use std::{
    error::Error,
    fs::File,
    io::{ self, Read, Write },
    os::raw::*,
};

use ayano::{
    Server, Request,
    StatusCode, ContentType, CacheControl,
};

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const INDEX: &[u8] = include_bytes!("../rsc/index.html");
const STYLES: &[u8] = include_bytes!("../rsc/styles.css");
const SCRIPTS: &[u8] = include_bytes!("../rsc/scripts.js");

const COMMAND_PLAY: &[u8] = b"cycle pause\n";
const COMMAND_MINUSCHAPTER: &[u8] = b"cycle chapter down\n";
const COMMAND_PLUSCHAPTER: &[u8] = b"cycle chapter up\n";
const COMMAND_MINUSPLAYLIST: &[u8] = b"playlist-prev\n";
const COMMAND_PLUSPLAYLIST: &[u8] = b"playlist-next\n";
const COMMAND_MINUS5: &[u8] = b"seek -5\n";
const COMMAND_PLUS5: &[u8] = b"seek 5\n";
const COMMAND_MINUS75: &[u8] = b"seek -75\n";
const COMMAND_PLUS75: &[u8] = b"seek 75\n";
const COMMAND_FULLSCREEN: &[u8] = b"cycle fullscreen\n";
const COMMAND_SUBTITLES: &[u8] = b"cycle sub\n";
const COMMAND_TITLE: &[u8] = b"show-text ${media-title} 5000\n";
const COMMAND_TIME: &[u8] = b"show-text \"${playback-time} (${time-remaining})\" 5000\n";
const COMMAND_QUIT: &[u8] = b"quit\n";

const PIPE_MAX_WAIT: c_ulong = 5_000;

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);
    
    if let Err(error) = process() {
        println!();
        println!("ERROR: {}", error);
    }
    
    println!();
    print!("Press 'enter' key to exit...");
    io::stdout().flush().ok();
    
    let _ = io::stdin().read(&mut [0]).ok();
}

fn process() -> Result<(), Box<dyn Error>> {
    let address = rin::get::<&str>(b"address")?;
    let server = Server::bind(address)?;
    
    println!();
    println!("Listening on http://{}", address);
    
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
        
        ("GET", "/") => index(request),
        ("GET", "/styles.css") => styles(request),
        ("GET", "/scripts.js") => scripts(request),
        
        // -------------------- commands --------------------
        
        ("POST", "/play") => send_command(request, COMMAND_PLAY),
        ("POST", "/minuschapter") => send_command(request, COMMAND_MINUSCHAPTER),
        ("POST", "/pluschapter") => send_command(request, COMMAND_PLUSCHAPTER),
        ("POST", "/minusplaylist") => send_command(request, COMMAND_MINUSPLAYLIST),
        ("POST", "/plusplaylist") => send_command(request, COMMAND_PLUSPLAYLIST),
        ("POST", "/minus5") => send_command(request, COMMAND_MINUS5),
        ("POST", "/plus5") => send_command(request, COMMAND_PLUS5),
        ("POST", "/minus75") => send_command(request, COMMAND_MINUS75),
        ("POST", "/plus75") => send_command(request, COMMAND_PLUS75),
        ("POST", "/fullscreen") => send_command(request, COMMAND_FULLSCREEN),
        ("POST", "/subtitles") => send_command(request, COMMAND_SUBTITLES),
        ("POST", "/title") => send_command(request, COMMAND_TITLE),
        ("POST", "/time") => send_command(request, COMMAND_TIME),
        ("POST", "/quit") => send_command(request, COMMAND_QUIT),
        
        // -------------------- not found --------------------
        
        _ => Err("Endpoint not found".into()),
        
    }
}

fn index(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
        .and_then(|mut response| response.write_all(INDEX))?;
    
    Ok(())
}

fn styles(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Css, CacheControl::Static)
        .and_then(|mut response| response.write_all(STYLES))?;
    
    Ok(())
}

fn scripts(request: &mut Request) -> Result<(), Box<dyn Error>> {
    request.start_response(StatusCode::Ok, ContentType::Javascript, CacheControl::Static)
        .and_then(|mut response| response.write_all(SCRIPTS))?;
    
    Ok(())
}

fn send_command(request: &mut Request, command: &[u8]) -> Result<(), Box<dyn Error>> {
    let pipe = rin::get::<&str>(b"pipe")?;
    let wide_pipe = chikuwa::win_string(pipe)?;
    
    unsafe {
        
        let result = sys::WaitNamedPipeW(
            wide_pipe.as_ptr(),
            PIPE_MAX_WAIT,
        );
        
        if result == 0 {
            return Err(Box::new(io::Error::last_os_error()));
        }
        
    }
    
    let mut file = File::options()
        .write(true)
        .open(pipe)?;
    
    file.write_all(command)?;
    
    request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
    
    Ok(())
}
