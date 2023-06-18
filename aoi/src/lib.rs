mod listener;

use std::{
    fs::{ OpenOptions, File },
    io::{ Read, Write, Error },
    net::TcpStream,
    os::raw::*,
    path::Path,
    thread,
    time::Duration,
};

use listener::{ Listener, ListenerStopper };

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

const INDEX: &str = include_str!("../rsc/index.html");
const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;
const PIPE_MAX_WAIT: c_ulong = 5000; // milliseconds

pub struct RemoteControlServer {
    stopper: ListenerStopper,
}

impl RemoteControlServer {
    
    // ---------- constructors ----------
    
    
    pub fn start<N: FnOnce(Error) + Send + 'static>(name: &Path, bind: &str, notify: N) -> Result<Self, Error> {
        unsafe {
            
            let result = ffi::WaitNamedPipeW(
                chikuwa::WinString::from(name).as_ptr(),
                PIPE_MAX_WAIT,
            );
            
            if result == 0 {
                return Err(Error::last_os_error());
            }
            
        }
        
        let pipe = OpenOptions::new()
            .write(true)
            .open(name)?;
        
        let (listener, stopper) = Listener::new(bind)?;
        
        Self::listen(listener, pipe, notify)?;
        
        Ok(Self {
            stopper,
        })
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn stop(&mut self) {
        self.stopper.stop();
    }
    
    
    // ---------- helpers ----------
    
    
    fn listen<N: FnOnce(Error) + Send + 'static>(listener: Listener, mut pipe: File, notify: N) -> Result<(), Error> {
        thread::Builder::new().spawn(move || {
            
            if let Err(error) = Self::handle_connections(&listener, &mut pipe) {
                notify(error);
            }
            
        })?;
        
        Ok(())
    }
    
    fn handle_connections(listener: &Listener, pipe: &mut File) -> Result<(), Error> {
        loop {
            
            let mut stream = listener.accept()?;
            
            stream.set_read_timeout(STREAM_TIMEOUT)?;
            stream.set_write_timeout(STREAM_TIMEOUT)?;
            
            let Some(request) = Self::get_request(&mut stream) else {
                continue;
            };
            
            // index
            
            if request.starts_with(b"GET / ") {
                Self::send_response(&mut stream, "200 OK", Some(INDEX)).ok();
                continue;
            }
            
            // command
            
            if let Some(command) = Self::get_command(&request) {
                
                if let Err(error) = pipe.write_all(command.as_bytes()) {
                    Self::send_response(&mut stream, "500 Internal Server Error", Some(&error.to_string())).ok();
                    return Err(error);
                }
                
                Self::send_response(&mut stream, "204 No Content", None).ok();
                continue;
                
            }
            
            // not found
            
            Self::send_response(&mut stream, "404 Not Found", Some("Endpoint not found")).ok();
            
        }
    }
    
    fn get_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
        let mut request = Vec::new();
        
        loop {
            
            let mut buffer = Vec::from([0; CONNECTION_BUFFER_SIZE]);
            
            let bytes = stream.read(&mut buffer).ok()?;
            
            if bytes == 0 {
                return None;
            }
            
            buffer.truncate(bytes);
            request.append(&mut buffer);
            
            if let Some(index) = request.windows(4).position(|curr| curr == b"\r\n\r\n") {
                // discard body, if any
                request.truncate(index + 4);
                break;
            }
            
        }
        
        Some(request)
    }
    
    fn get_command(bytes: &[u8]) -> Option<&str> {
        match bytes {
            _ if bytes.starts_with(b"GET /play ") => Some("cycle pause\n"),
            _ if bytes.starts_with(b"GET /minuschapter ") => Some("cycle chapter down\n"),
            _ if bytes.starts_with(b"GET /pluschapter ") => Some("cycle chapter up\n"),
            _ if bytes.starts_with(b"GET /minusplaylist ") => Some("playlist-prev\n"),
            _ if bytes.starts_with(b"GET /plusplaylist ") => Some("playlist-next\n"),
            _ if bytes.starts_with(b"GET /minus5 ") => Some("seek -5\n"),
            _ if bytes.starts_with(b"GET /plus5 ") => Some("seek 5\n"),
            _ if bytes.starts_with(b"GET /minus75 ") => Some("seek -75\n"),
            _ if bytes.starts_with(b"GET /plus75 ") => Some("seek 75\n"),
            _ if bytes.starts_with(b"GET /fullscreen ") => Some("cycle fullscreen\n"),
            _ if bytes.starts_with(b"GET /subtitles ") => Some("cycle sub\n"),
            _ if bytes.starts_with(b"GET /title ") => Some("show-text ${media-title} 5000\n"),
            _ if bytes.starts_with(b"GET /time ") => Some("show-text \"${playback-time} (${time-remaining})\" 5000\n"),
            _ => None,
        }
    }
    
    fn send_response(stream: &mut TcpStream, status: &str, body: Option<&str>) -> Result<(), Error> {
        let mut response = Vec::new();
        
        match body {
            
            Some(payload) => {
                
                response.reserve(50 + status.len() + (payload.len().ilog10() + 1) as usize + payload.len());
                
                write!(response, "HTTP/1.0 {}\r\n", status).unwrap();
                write!(response, "Connection: close\r\n").unwrap();
                write!(response, "Content-Length: {}\r\n", payload.len()).unwrap();
                write!(response, "\r\n").unwrap();
                write!(response, "{}", payload).unwrap();
                
            },
            
            None => {
                
                response.reserve(50 + status.len() + 1);
                
                write!(response, "HTTP/1.0 {}\r\n", status).unwrap();
                write!(response, "Connection: close\r\n").unwrap();
                write!(response, "Content-Length: 0\r\n").unwrap();
                write!(response, "\r\n").unwrap();
                
            },
            
        }
        
        stream.write_all(&response)
    }
    
}
