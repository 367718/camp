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

const INDEX: &[u8] = include_bytes!("../rsc/index.html");
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
            
            let mut parts = request.split(|&curr| curr == b' ');
            
            let (Some(method), Some(path)) = (parts.next(), parts.next()) else {
                continue;
            };
            
            // index
            
            if method == b"GET" && path == b"/" {
                Self::send_response(&mut stream, b"200 OK", Some(INDEX)).ok();
                continue;
            }
            
            // command
            
            if let Some(command) = Self::get_command(method, path) {
                
                if let Err(error) = pipe.write_all(command) {
                    Self::send_response(&mut stream, b"500 Internal Server Error", Some(error.to_string().as_bytes())).ok();
                    return Err(error);
                }
                
                Self::send_response(&mut stream, b"200 OK", None).ok();
                continue;
                
            }
            
            // not found
            
            Self::send_response(&mut stream, b"404 Not Found", Some(b"Endpoint not found")).ok();
            
        }
    }
    
    fn get_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
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
        
        Some(request)
    }
    
    fn get_command(method: &[u8], path: &[u8]) -> Option<&'static [u8]> {
        if method != b"GET" {
            return None;
        }
        
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
    
    fn send_response(stream: &mut TcpStream, status: &[u8], body: Option<&[u8]>) -> Result<(), Error> {
        stream.write_all(b"HTTP/1.0 ")?;
        stream.write_all(status)?;
        stream.write_all(b"\r\n")?;
        
        stream.write_all(b"Connection: close\r\n")?;
        
        match body {
            
            Some(payload) => {
                stream.write_all(b"Content-Length: ")?;
                stream.write_all(payload.len().to_string().as_bytes())?;
                stream.write_all(b"\r\n")?;
                stream.write_all(b"\r\n")?;
                stream.write_all(payload)?;
            },
            
            None => {
                stream.write_all(b"Content-Length: 0\r\n")?;
                stream.write_all(b"\r\n")?;
            },
            
        }
        
        Ok(())
    }
    
}
