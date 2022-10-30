mod listener;

use std::{
    fs::File,
    io::{ Read, Write, Error },
    net::TcpStream,
    os::windows::ffi::OsStrExt,
    path::Path,
    thread,
    time::Duration,
};

use listener::{ Listener, ListenerStopper };

mod ffi {
    
    extern "system" {
        
        // https://docs.microsoft.com/en-us/windows/win32/api/namedpipeapi/nf-namedpipeapi-waitnamedpipew
        pub fn WaitNamedPipeW(
            lpNamedPipeName: *const u16, // LPCWSTR -> *const WCHAR -> wchar_t
            nTimeOut: u32, // DWORD -> c_ulong
        ) -> i32; // BOOL -> c_int
        
    }
    
}

const INDEX: &str = include_str!("../rsc/index.html");
const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;
const PIPE_MAX_WAIT: u32 = 5000; // milliseconds

pub struct RemoteControlServer {
    stopper: ListenerStopper,
}

impl RemoteControlServer {
    
    // ---------- constructors ----------
    
    
    pub fn start<N: FnOnce(Error) + Send + 'static>(pipe: &Path, bind: &str, notify: N) -> Result<Self, Error> {
        let encoded_path: Vec<u16> = pipe.as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        let available = unsafe {
            
            ffi::WaitNamedPipeW(
                encoded_path.as_ptr(),
                PIPE_MAX_WAIT,
            )
            
        };
        
        if available == 0 {
            return Err(Error::last_os_error());
        }
        
        let pipe = File::create(pipe)?;
        let (listener, stopper) = Listener::new(bind)?;
        
        Self::listen(pipe, listener, notify)?;
        
        Ok(Self {
            stopper,
        })
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn stop(&mut self) {
        self.stopper.stop();
    }
    
    
    // ---------- helpers ----------
    
    
    fn listen<N: FnOnce(Error) + Send + 'static>(mut pipe: File, listener: Listener, notify: N) -> Result<(), Error> {
        thread::Builder::new().spawn(move || {
            
            if let Err(error) = Self::handle_connections(&mut pipe, &listener) {
                notify(error);
            }
            
        })?;
        
        Ok(())
    }
    
    fn handle_connections(pipe: &mut File, listener: &Listener) -> Result<(), Error> {
        loop {
            
            let mut stream = listener.accept()?;
            
            stream.set_read_timeout(STREAM_TIMEOUT)?;
            stream.set_write_timeout(STREAM_TIMEOUT)?;
            
            if let Some(request) = Self::get_request(&mut stream) {
                
                if let Some(command) = Self::get_command(&request) {
                    
                    if let Err(error) = pipe.write_all(command.as_bytes()) {
                        Self::send_response(&mut stream, "500 Internal Server Error", &error.to_string()).ok();
                        return Err(error);
                    }
                    
                }
                
                // always send index if no error ocurred
                Self::send_response(&mut stream, "200 OK", INDEX).ok();
                
            }
            
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
            _ if bytes.starts_with(b"GET /play? ") => Some("cycle pause\n"),
            _ if bytes.starts_with(b"GET /minuschapter? ") => Some("cycle chapter down\n"),
            _ if bytes.starts_with(b"GET /pluschapter? ") => Some("cycle chapter up\n"),
            _ if bytes.starts_with(b"GET /minusplaylist? ") => Some("playlist-prev\n"),
            _ if bytes.starts_with(b"GET /plusplaylist? ") => Some("playlist-next\n"),
            _ if bytes.starts_with(b"GET /minus5? ") => Some("seek -5\n"),
            _ if bytes.starts_with(b"GET /plus5? ") => Some("seek 5\n"),
            _ if bytes.starts_with(b"GET /minus85? ") => Some("seek -85\n"),
            _ if bytes.starts_with(b"GET /plus85? ") => Some("seek 85\n"),
            _ if bytes.starts_with(b"GET /fullscreen? ") => Some("cycle fullscreen\n"),
            _ if bytes.starts_with(b"GET /subtitles? ") => Some("cycle sub\n"),
            _ if bytes.starts_with(b"GET /title? ") => Some("show-text ${media-title} 5000\n"),
            _ if bytes.starts_with(b"GET /time? ") => Some("show-text \"${playback-time} (${time-remaining})\" 5000\n"),
            _ => None,
        }
    }
    
    fn send_response(stream: &mut TcpStream, status: &str, body: &str) -> Result<(), Error> {
        // final number represents content length
        let mut response = Vec::with_capacity(50 + status.len() + body.len() + 4);
        
        write!(response, "HTTP/1.0 {}\r\n", status).unwrap();
        write!(response, "Connection: close\r\n").unwrap();
        write!(response, "Content-Length: {}\r\n", body.len()).unwrap();
        write!(response, "\r\n").unwrap();
        write!(response, "{}", body).unwrap();
        
        stream.write_all(&response)
    }
    
}
