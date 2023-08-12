use std::{
    error::Error,
    io::{ self, ErrorKind, Read, Write, Take },
    net::{ TcpStream, ToSocketAddrs },
    str,
    time::{ Instant, Duration },
};

use schannel::{
    schannel_cred::{ SchannelCred, Direction },
    tls_stream::{ TlsStream, Builder },
};

pub struct Entry {
    host: String,
    port: u16,
    connection: Connection,
}

enum Connection {
    Http(TcpStream),
    Https(TlsStream<TcpStream>),
}

struct ReadHandle<'c> {
    inner: Take<&'c mut Connection>,
}

const RESPONSE_SIZE_LIMIT: u64 = 50 * 1024 * 1024 + 1;
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;

impl Entry {
    
    // ---------- constructors ----------
    
    
    pub fn new(host: &str, port: u16, secure: bool, timeout: Duration) -> Result<Self, Box<dyn Error>> {
        let connection = Self::connect(host, port, secure, timeout)?;
        
        Ok(Self {
            host: host.to_string(),
            port,
            connection,
        })
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn is_connection_already_open(&self, host: &str, port: u16, secure: bool) -> bool {
        self.host == host && self.port == port && matches!(self.connection, Connection::Https(_)) == secure
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn reopen_connection_if_needed(&mut self, timeout: Duration) -> Result<(), Box<dyn Error>> {
        
        fn can_be_reused(connection: &Connection) -> Result<bool, Box<dyn Error>> {
            let mut reusable = false;
            
            let stream = match connection {
                Connection::Http(stream) => stream,
                Connection::Https(tls) => tls.get_ref(),
            };
            
            stream.set_nonblocking(true)?;
            
            // the only acceptable condition here is a "WouldBlock" error, since an "Ok" would mean
            // that bytes were read past the last request body and an EOF would mean the connection was closed
            if let Err(error) = stream.peek(&mut [0; 1]) {
                reusable = error.kind() == ErrorKind::WouldBlock;
            }
            
            stream.set_nonblocking(false)?;
            
            Ok(reusable)
        }
        
        if ! can_be_reused(&self.connection).unwrap_or(false) {
            self.connection = Self::connect(&self.host, self.port, matches!(self.connection, Connection::Https(_)), timeout)?;
        }
        
        Ok(())
        
    }
    
    pub fn send_request(&mut self, path: &str) -> Result<(Vec<u8>, bool), Box<dyn Error>> {
        // ----- build request -----
        
        let mut request: Vec<u8> = Vec::with_capacity(96 + path.len() + self.host.len() + (self.port.ilog10() + 1) as usize);
        
        write!(request, "GET {} HTTP/1.0\r\n", path).unwrap();
        write!(request, "Host: {}:{}\r\n", self.host, self.port).unwrap();
        write!(request, "Connection: keep-alive\r\n").unwrap();
        write!(request, "Accept-Encoding: identity\r\n").unwrap();
        write!(request, "Content-length: 0\r\n").unwrap();
        write!(request, "\r\n").unwrap();
        
        // ----- write request -----
        
        self.connection.write_all(&request)?;
        
        // ----- sections -----
        
        let mut headers = Vec::new();
        let mut body = Vec::new();
        
        // ----- build handle -----
        
        let mut handle = ReadHandle::new(&mut self.connection);
        
        // ----- read response headers -----
        
        Self::fill_headers(&mut handle, &mut headers, &mut body)?;
        
        // ----- check status code and keep-alive -----
        
        let status_code = headers.split(|&curr| curr == b' ')
            .nth(1)
            .ok_or("Bad response status code")?;
        
        if status_code != b"200" {
            return Err(str::from_utf8(status_code)?.into());
        }
        
        let keep_alive = chikuwa::tag_range(&headers, b"Connection: ", b"\r\n")
            .map(|range| &headers[range])
            .filter(|value| value == b"keep-alive")
            .is_some();
        
        // ----- read response body -----
        
        Self::fill_body(&mut handle, &headers, &mut body, keep_alive)?;
        
        Ok((body, keep_alive))
    }
    
    
    // ---------- helpers ----------
    
    
    fn connect(host: &str, port: u16, secure: bool, timeout: Duration) -> Result<Connection, Box<dyn Error>> {
        let start = Instant::now()
            .checked_add(timeout)
            .ok_or("Bad timeout")?;
        
        for address in (host, port).to_socket_addrs()? {
            
            let current = start.checked_duration_since(Instant::now())
                .ok_or("DNS resolution exceeded the specified timeout or no connection could be established in time")?;
            
            if let Ok(stream) = TcpStream::connect_timeout(&address, current) {
                
                stream.set_read_timeout(Some(timeout))?;
                stream.set_write_timeout(Some(timeout))?;
                
                if secure {
                    
                    let cred = SchannelCred::builder().acquire(Direction::Outbound)?;
                    let tls = Builder::new().domain(host).connect(cred, stream)?;
                    
                    return Ok(Connection::Https(tls));
                    
                }
                
                return Ok(Connection::Http(stream));
                
            }
            
        }
        
        Err("Connection to remote host failed".into())
    }
    
    fn fill_headers(handle: &mut ReadHandle, headers: &mut Vec<u8>, body: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
        // read until body is found
        
        let mut buffer = vec![0; CONNECTION_BUFFER_SIZE];
        
        loop {
            
            let bytes = handle.read(&mut buffer)?;
            
            if bytes == 0 {
                return Err("Connection interrupted".into());
            }
            
            headers.extend_from_slice(&buffer[..bytes]);
            
            // separate body
            if let Some(index) = headers.windows(4).position(|curr| curr == b"\r\n\r\n") {
                body.append(&mut headers.split_off(index + 4));
                break;
            }
            
        }
        
        Ok(())
    }
    
    fn fill_body(handle: &mut ReadHandle, headers: &[u8], body: &mut Vec<u8>, keep_alive: bool) -> Result<(), Box<dyn Error>> {
        if keep_alive {
            
            // Connection: keep-alive
            // read until the amount of bytes specified in header is received
            
            let content_length = chikuwa::tag_range(headers, b"Content-Length: ", b"\r\n")
                .map(|range| &headers[range])
                .and_then(|value| str::from_utf8(value).ok())
                .and_then(|value| value.parse::<usize>().ok())
                .ok_or("Could not determine body length")?;
            
            if content_length > 0 {
                
                let mut buffer = [0; CONNECTION_BUFFER_SIZE];
                
                body.reserve(content_length);
                
                while body.len() < content_length {
                    
                    let bytes = handle.read(&mut buffer)?;
                    
                    if bytes == 0 {
                        return Err("Connection interrupted".into());
                    }
                    
                    body.extend_from_slice(&buffer[..bytes]);
                    
                }
                
            }
            
            body.truncate(content_length);
        
        } else {
            
            // Connection: close
            // read until EOF
            
            let mut buffer = Vec::new();
            handle.read_to_end(&mut buffer)?;
            body.append(&mut buffer);
            
        }
        
        Ok(())
    }
    
}

impl Write for Connection {
    
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Connection::Http(stream) => stream.write(buf),
            Connection::Https(tls) => tls.write(buf),
        }
    }
    
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Connection::Http(stream) => stream.flush(),
            Connection::Https(tls) => tls.flush(),
        }
    }
    
}

impl Read for Connection {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Connection::Http(stream) => stream.read(buf),
            Connection::Https(tls) => tls.read(buf),
        }
    }
    
}

impl<'c> ReadHandle<'c> {
    
    pub fn new(connection: &'c mut Connection) -> Self {
        Self {
            inner: connection.take(RESPONSE_SIZE_LIMIT),
        }
    }
    
}

impl Read for ReadHandle<'_> {
    
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes = self.inner.read(buf)?;
        
        if self.inner.limit() == 0 {
            return Err(io::Error::new(ErrorKind::Other, "Maximum response size reached"));
        }
        
        Ok(bytes)
    }
    
}
