use std::{
    error::Error,
    fs::{ OpenOptions, File },
    io::{ self, Read, Write },
    os::{
        raw::*,
        windows::io::{ AsRawHandle, FromRawHandle },
    },
};

use ayano::{ Server, StatusCode, ContentType, CacheControl };

mod ffi {
    
    use super::*;
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew
        pub fn WaitNamedPipeW(
            lpNamedPipeName: *const c_ushort,
            nTimeOut: c_ulong,
        ) -> c_int;
        
    }
    
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

const INDEX: &[u8] = include_bytes!("../rsc/index.html");
const PIPE_MAX_WAIT: c_ulong = 5000; // milliseconds

fn main() {
    // avoid locking
    let mut stdout = unsafe {
        
        File::from_raw_handle(io::stdout().as_raw_handle())
        
    };
    
    write!(&mut stdout, "{} v{}", APP_NAME, APP_VERSION).unwrap();
    
    if let Err(error) = process(&mut stdout) {
        write!(&mut stdout, "\n\n{}", error).unwrap();
    }
    
    stdout.write_all(b"\n\nPress 'enter' key to exit...").unwrap();
    
    let _ = io::stdin().read(&mut [0u8]).unwrap();
}

fn process(stdout: &mut File) -> Result<(), Box<dyn Error>> {
    // -------------------- config --------------------
    
    stdout.write_all(b"\n\nLoading configuration...").unwrap();
    
    let address = rin::get(b"address")?;
    let name = rin::get(b"name")?;
    
    // -------------------- pipe --------------------
    
    stdout.write_all(b"\nConnecting to named pipe...").unwrap();
    
    unsafe {
        
        let result = ffi::WaitNamedPipeW(
            chikuwa::WinString::from(name).as_ptr(),
            PIPE_MAX_WAIT,
        );
        
        if result == 0 {
            return Err(io::Error::last_os_error().into());
        }
        
    }
    
    let mut pipe = OpenOptions::new()
        .write(true)
        .open(name)?;
    
    // -------------------- listener --------------------
    
    stdout.write_all(b"\nBinding address...").unwrap();
    
    let server = Server::new(address)?;
    
    write!(stdout, "\n\nListening on {}", address).unwrap();
    
    // -------------------- requests --------------------
    
    for mut request in server {
        
        if let Err(error) = handle_request(&mut request, &mut pipe) {
            request.start_response(StatusCode::Error, ContentType::Plain, CacheControl::Dynamic)
                .and_then(|mut response| response.write_all(error.to_string().as_bytes()))
                .ok();
        }
        
    }
    
    Ok(())
}

fn handle_request(request: &mut ayano::Request, pipe: &mut File) -> Result<(), Box<dyn Error>> {
    let (method, endpoint) = request.resource();
    
    if method != b"GET" {
        return Err("Endpoint not found".into());
    }
    
    // -------------------- index --------------------
    
    if endpoint == b"/" {
        
        request.start_response(StatusCode::Ok, ContentType::Html, CacheControl::Static)
            .and_then(|mut response| response.write_all(INDEX))?;
        
        return Ok(());
        
    }
    
    // -------------------- commands --------------------
    
    if let Some(command) = get_command(endpoint) {
        
        pipe.write_all(command)?;
        
        request.start_response(StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic)
            .and_then(|mut response| response.write_all(b"200 OK"))?;
        
        return Ok(());
        
    }
    
    // -------------------- not found --------------------
    
    Err("Endpoint not found".into())
}

fn get_command(endpoint: &[u8]) -> Option<&'static [u8]> {
    match endpoint {
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
        _ => None,
    }
}
