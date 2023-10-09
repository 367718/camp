use std::{
    error::Error,
    fs::{ OpenOptions, File },
    io::{ self, stdout, Read, Write, BufWriter },
    net::{ TcpListener, TcpStream },
    os::raw::*,
    time::Duration,
};

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
const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;
const PIPE_MAX_WAIT: c_ulong = 5000; // milliseconds

fn main() {
    println!("{} v{}", APP_NAME, APP_VERSION);
    println!("--------------------");
    
    if let Err(error) = process() {
        println!();
        println!("ERROR: {}", error);
    }
    
    println!();
    print!("Press 'enter' key to exit...");
    
    stdout().flush().unwrap();
    io::stdin().read_line(&mut String::new()).unwrap();
}

fn process() -> Result<(), Box<dyn Error>> {
    println!();
    println!("Loading configuration file...");
    
    let config = rin::Config::load()?;
    let address = config.get(b"address")?;
    let name = config.get(b"name")?;
    
    println!();
    println!("Connecting to named pipe...");
    
    unsafe {
        
        let result = ffi::WaitNamedPipeW(
            chikuwa::WinString::from(name).as_ptr(),
            PIPE_MAX_WAIT,
        );
        
        if result == 0 {
            return Err(io::Error::last_os_error().into());
        }
        
    }
    
    let pipe = OpenOptions::new()
        .write(true)
        .open(name)?;
    
    println!();
    println!("Binding listener...");
    
    let listener = TcpListener::bind(address)?;
    
    println!();
    println!("Success!");
    
    println!();
    println!("Listening on {}", address);
    
    listen(&listener, pipe)
}

fn listen(listener: &TcpListener, mut pipe: File) -> Result<(), Box<dyn Error>> {
    for stream in listener.incoming() {
        
        let mut stream = stream?;
        
        stream.set_read_timeout(STREAM_TIMEOUT)?;
        stream.set_write_timeout(STREAM_TIMEOUT)?;
        
        let Some((method, path)) = get_request(&mut stream) else {
            continue;
        };
        
        if method != b"GET" {
            continue;
        }
        
        // index
        
        if path == b"/" {
            send_response(stream, b"200 OK", Some(INDEX)).ok();
            continue;
        }
        
        // command
        
        if let Some(command) = get_command(&path) {
            
            if let Err(error) = pipe.write_all(command) {
                send_response(stream, b"500 Internal Server Error", Some(error.to_string().as_bytes())).ok();
                return Err(error.into());
            }
            
            send_response(stream, b"200 OK", None).ok();
            continue;
            
        }
        
        // not found
        
        send_response(stream, b"404 Not Found", Some(b"Endpoint not found")).ok();
        
    }
    
    Ok(())
}

fn get_request(stream: &mut TcpStream) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut request = Vec::new();
    let mut buffer = [0; CONNECTION_BUFFER_SIZE];
    
    loop {
        
        let bytes = stream.read(&mut buffer).ok()?;
        
        if bytes == 0 {
            return None;
        }
        
        request.extend_from_slice(&buffer[..bytes]);
        
        if let Some(index) = request.windows(4).position(|curr| curr == b"\r\n\r\n") {
            // discard body, if any
            request.truncate(index + 4);
            break;
        }
        
    }
    
    let mut parts = request.split(|&curr| curr == b' ');
    
    let method = parts.next()?.to_vec();
    let path = parts.next()?.to_vec();
    
    Some((method, path))
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
        _ => None,
    }
}

fn send_response(stream: TcpStream, status: &[u8], payload: Option<&[u8]>) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(stream);
    
    writer.write_all(b"HTTP/1.0 ")?;
    writer.write_all(status)?;
    writer.write_all(b"\r\n")?;
    
    writer.write_all(b"Connection: close\r\n")?;
    
    if let Some(payload) = payload {
        writer.write_all(b"Content-Length: ")?;
        writer.write_all(payload.len().to_string().as_bytes())?;
        writer.write_all(b"\r\n")?;
        writer.write_all(b"\r\n")?;
        writer.write_all(payload)?;
    } else {
        writer.write_all(b"Content-Length: 0\r\n")?;
        writer.write_all(b"\r\n")?;
    }
    
    Ok(())
}
