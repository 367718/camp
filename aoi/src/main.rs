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
const STYLES: &[u8] = include_bytes!("../rsc/styles.css");
const SCRIPTS: &[u8] = include_bytes!("../rsc/scripts.js");

const PIPE_MAX_WAIT: c_ulong = 5000; // milliseconds

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
    let address = rin::get(b"address")?;
    let mut server = Server::bind(address)?;
    
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
    let (method, path) = request.method_and_path()
        .ok_or("Invalid request")?;
    
    if method == b"GET" {
        
        // -------------------- index --------------------
        
        if path == b"/" {
            
            request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
                .and_then(|mut response| response.write_all(INDEX))?;
            
            return Ok(());
            
        }
        
        // -------------------- styles --------------------
        
        if path == b"/styles.css" {
            
            request.start_response(StatusCode::Ok, ContentType::Css, CacheControl::Static)
                .and_then(|mut response| response.write_all(STYLES))?;
            
            return Ok(());
            
        }
        
        // -------------------- scripts --------------------
        
        if path == b"/scripts.js" {
            
            request.start_response(StatusCode::Ok, ContentType::Javascript, CacheControl::Static)
                .and_then(|mut response| response.write_all(SCRIPTS))?;
            
            return Ok(());
            
        }
        
        // -------------------- commands --------------------
        
        if let Some(command) = get_command(path) {
            
            let pipe = rin::get(b"pipe")?;
            write_to_named_pipe(pipe, command)?;
            request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)?;
            
            return Ok(());
            
        }
        
    }
    
    // -------------------- not found --------------------
    
    Err("Endpoint not found".into())
}

fn get_command(path: &[u8]) -> Option<&'static [u8]> {
    let result: &[u8] = match path {
        b"/play" => b"cycle pause\n",
        b"/minuschapter" => b"cycle chapter down\n",
        b"/pluschapter" => b"cycle chapter up\n",
        b"/minusplaylist" => b"playlist-prev\n",
        b"/plusplaylist" => b"playlist-next\n",
        b"/minus5" => b"seek -5\n",
        b"/plus5" => b"seek 5\n",
        b"/minus75" => b"seek -75\n",
        b"/plus75" => b"seek 75\n",
        b"/fullscreen" => b"cycle fullscreen\n",
        b"/subtitles" => b"cycle sub\n",
        b"/title" => b"show-text ${media-title} 5000\n",
        b"/time" => b"show-text \"${playback-time} (${time-remaining})\" 5000\n",
        b"/quit" => b"quit\n",
        _ => return None,
    };
    
    Some(result)
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
    
    let mut pipe = File::options()
        .write(true)
        .open(path)?;
    
    pipe.write_all(data)
}
