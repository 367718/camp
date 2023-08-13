use std::{
    error::Error,
    io::{ self, ErrorKind, Read, Write, BufWriter, Take },
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
        // request
        
        Self::send_request(&mut self.connection, path, &self.host, self.port)?;
        
        // response
        
        let mut headers = Vec::new();
        let mut body = Vec::new();
        
        let mut reader = (&mut self.connection).take(RESPONSE_SIZE_LIMIT);
        
        // headers
        
        Self::fill_headers(&mut reader, &mut headers, &mut body)?;
        
        // status code
        
        let status_code = headers.split(|&curr| curr == b' ')
            .nth(1)
            .ok_or("Bad response status code")?;
        
        if status_code != b"200" {
            return Err(str::from_utf8(status_code)?.into());
        }
        
        // keep-alive
        
        let field = b"\r\nConnection: keep-alive\r\n";
        
        let keep_alive = headers.windows(field.len())
            .any(|window| window.eq_ignore_ascii_case(field));
        
        // body
        
        Self::fill_body(&mut reader, &headers, &mut body, keep_alive)?;
        
        Ok((body, keep_alive))
    }
    
    
    // ---------- helpers ----------
    
    
    fn send_request(connection: &mut Connection, path: &str, host: &str, port: u16) -> Result<(), Box<dyn Error>> {
        let mut writer = BufWriter::new(connection);
        
        writer.write_all(b"GET ")?;
        writer.write_all(path.as_bytes())?;
        writer.write_all(b" HTTP/1.0\r\n")?;
        
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
    
    fn fill_headers(reader: &mut Take<&mut Connection>, headers: &mut Vec<u8>, body: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
        // read until body is found
        
        let mut buffer = [0; CONNECTION_BUFFER_SIZE];
        
        loop {
            
            let bytes = reader.read(&mut buffer)?;
            
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
    
    fn fill_body(reader: &mut Take<&mut Connection>, headers: &[u8], body: &mut Vec<u8>, keep_alive: bool) -> Result<(), Box<dyn Error>> {
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
                    
                    let bytes = reader.read(&mut buffer)?;
                    
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
            reader.read_to_end(&mut buffer)?;
            body.append(&mut buffer);
            
        }
        
        Ok(())
    }
    
}

impl PartialEq<(&str, u16, bool)> for Entry {
    
    fn eq(&self, other: &(&str, u16, bool)) -> bool {
        self.host == other.0 && self.port == other.1 && matches!(self.connection, Connection::Https(_)) == other.2
    }
    
}

impl Connection {
    
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
