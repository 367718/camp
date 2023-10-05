use std::{
    error::Error,
    io::{ self, ErrorKind, Read, Write, BufWriter },
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

const RESPONSE_SIZE_LIMIT: u64 = 50 * 1024 * 1024 + 1;
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;

impl Entry {
    
    // ---------- constructors ----------
    
    
    pub fn new(host: &str, port: u16, secure: bool, timeout: Duration) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            host: host.to_string(),
            port,
            connection: Connection::new(host, port, secure, timeout)?,
        })
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn can_be_reused(&self) -> bool {
        self.connection.can_be_reused().unwrap_or(false)
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn body(&mut self, path: &str) -> Result<(Vec<u8>, bool), Box<dyn Error>> {
        self.connection.send_request(path, &self.host, self.port)?;
        self.connection.get_response()
    }
    
}

impl PartialEq<(&str, u16, bool)> for Entry {
    
    fn eq(&self, other: &(&str, u16, bool)) -> bool {
        self.host == other.0 && self.port == other.1 && matches!(self.connection, Connection::Https(_)) == other.2
    }
    
}

impl Connection {
    
    // ---------- constructors ----------
    
    
    pub fn new(host: &str, port: u16, secure: bool, timeout: Duration) -> Result<Self, Box<dyn Error>> {
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
                    return Ok(Self::Https(tls));
                }
                
                return Ok(Self::Http(stream));
                
            }
            
        }
        
        Err("Connection to remote host failed".into())
    }
    
    
    // ---------- accessors ----------
    
    
    pub fn can_be_reused(&self) -> Result<bool, Box<dyn Error>> {
        let mut reusable = false;
        
        let stream = match self {
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
    
    
    // ---------- mutators ----------
    
    
    fn send_request(&mut self, path: &str, host: &str, port: u16) -> Result<(), Box<dyn Error>> {
        let mut writer = BufWriter::new(self);
        
        writer.write_all(b"GET ")?;
        writer.write_all(path.as_bytes())?;
        writer.write_all(b" HTTP/1.1\r\n")?;
        
        writer.write_all(b"Host: ")?;
        writer.write_all(host.as_bytes())?;
        writer.write_all(b":")?;
        writer.write_all(port.to_string().as_bytes())?;
        writer.write_all(b"\r\n")?;
        
        writer.write_all(b"Connection: keep-alive\r\n")?;
        writer.write_all(b"Accept-Encoding: identity\r\n")?;
        writer.write_all(b"Content-length: 0\r\n")?;
        writer.write_all(b"\r\n")?;
        
        Ok(())
    }
    
    fn get_response(&mut self) -> Result<(Vec<u8>, bool), Box<dyn Error>> {
        let mut reader = (self).take(RESPONSE_SIZE_LIMIT);
        
        let mut headers = Vec::new();
        let mut body = Vec::new();
        
        // ---------- headers ----------
        
        {
            
            let mut buffer = [0; CONNECTION_BUFFER_SIZE];
            
            loop {
                
                let bytes = reader.read(&mut buffer)
                    .ok()
                    .filter(|&bytes| bytes > 0)
                    .ok_or("Connection interrupted")?;
                
                headers.extend_from_slice(&buffer[..bytes]);
                
                // separate body
                if let Some(position) = headers.windows(4).position(|curr| curr == b"\r\n\r\n") {
                    let index = position.checked_add(4).ok_or("Response size exceeds acceptable values")?;
                    body.append(&mut headers.split_off(index));
                    headers.truncate(index);
                    break;
                }
                
            }
            
        }
        
        // status code
        
        let status_code = headers.split(|&curr| curr == b' ')
            .nth(1)
            .ok_or("Bad response status code")?;
        
        if status_code != b"200" {
            return Err(str::from_utf8(status_code)?.into());
        }
        
        // keep-alive
        
        let keep_alive = chikuwa::tag_range(&headers, b"\r\nConnection: ", b"\r\n")
            .map(|range| &headers[range])
            .filter(|value| value == b"keep-alive")
            .is_some();
        
        // chunked
        
        let chunked = chikuwa::tag_range(&headers, b"\r\nTransfer-Encoding: ", b"\r\n")
            .map(|range| &headers[range])
            .filter(|value| value == b"chunked")
            .is_some();
        
        // ---------- body ----------
        
        {
            
            if chunked {
                
                let mut buffer = [0; CONNECTION_BUFFER_SIZE];
                
                // read until empty chunk is found
                
                loop {
                    
                    if let Some(position) = body.windows(7).position(|curr| curr == b"\r\n0\r\n\r\n") {
                        let index = position.checked_add(7).ok_or("Response size exceeds acceptable values")?;
                        body.truncate(index);
                        break;
                    }
                    
                    let bytes = reader.read(&mut buffer)
                        .ok()
                        .filter(|&bytes| bytes > 0)
                        .ok_or("Connection interrupted")?;
                    
                    body.extend_from_slice(&buffer[..bytes]);
                    
                }
                
                // decode body
                
                let mut decoded = Vec::with_capacity(body.len());
                let mut remaining = body.as_slice();
                
                loop {
                    
                    let chunk_length_position = remaining
                        .windows(2)
                        .position(|curr| curr == b"\r\n")
                        .ok_or("Invalid chunk size")?;
                    
                    let chunk_length = str::from_utf8(&remaining[..chunk_length_position]).ok()
                        .and_then(|characters| usize::from_str_radix(characters, 16).ok())
                        .ok_or("Invalid chunk size")?;
                    
                    if chunk_length == 0 {
                        break;
                    }
                    
                    // skip chunk length and carriage return
                    remaining = &remaining[chunk_length_position + 2..];
                    
                    decoded.extend_from_slice(remaining.get(..chunk_length).ok_or("Invalid response")?);
                    
                    // skip carriage return
                    remaining = &remaining[chunk_length + 2..];
                    
                }
                
                Ok((decoded, keep_alive))
                
            } else {
                
                let mut buffer = [0; CONNECTION_BUFFER_SIZE];
                
                // read until the amount of bytes specified in header is received
                
                let content_length = chikuwa::tag_range(&headers, b"\r\nContent-Length: ", b"\r\n")
                    .map(|range| &headers[range])
                    .and_then(|value| str::from_utf8(value).ok())
                    .and_then(|value| value.parse::<usize>().ok())
                    .ok_or("Could not determine content length")?;
                
                if body.len() < content_length {
                    
                    body.reserve(content_length);
                    
                    while body.len() < content_length {
                        
                        let bytes = reader.read(&mut buffer)?;
                        
                        if bytes == 0 {
                            return Err("Connection interrupted".into());
                        }
                        
                        body.extend_from_slice(&buffer[..bytes]);
                        
                    }
                    
                }
                
                body.truncate(content_length);
                
                Ok((body, keep_alive))
                
            }
            
        }
        
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
