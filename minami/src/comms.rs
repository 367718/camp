use std::{
    error::Error,
    io::{ self, Read, Write, BufWriter },
    net::TcpStream,
    str,
    time::Duration,
};

const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;
const REQUEST_SIZE_LIMIT: u64 = 50 * 1024 * 1024 + 1;

#[derive(Copy, Clone)]
pub enum Status {
    Ok,
    Error,
    NotFound,
}

#[derive(Copy, Clone)]
pub enum ContentType {
    Plain,
    Html,
    Css,
    Javascript,
    Favicon,
}

pub struct Request {
    headers: Vec<u8>,
    body: Vec<u8>,
    stream: Option<TcpStream>,
}

struct Payload<'h, 'b> {
    boundary: &'h [u8],
    content: &'b [u8],
}

pub struct Response {
    writer: BufWriter<TcpStream>,
}

impl Status {
    
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Ok => b"200 OK",
            Self::NotFound => b"404 Not Found",
            Self::Error => b"500 Internal Server Error",
        }
    }
    
}

impl ContentType {
    
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Plain => b"text/plain; charset=utf-8",
            Self::Html => b"text/html; charset=utf-8",
            Self::Css => b"text/css; charset=utf-8",
            Self::Javascript => b"text/javascript; charset=utf-8",
            Self::Favicon => b"image/x-icon",
        }
    }
    
    pub fn cache_policy(self) -> &'static [u8] {
        #[allow(clippy::match_same_arms)]
        match self {
            Self::Plain => b"no-cache, no-store",
            Self::Html => b"no-cache, no-store",
            Self::Css => b"max-age=15552000, immutable",
            Self::Javascript => b"max-age=15552000, immutable",
            Self::Favicon => b"max-age=15552000, immutable",
        }
    }
    
}

impl Request {
    
    pub fn get(stream: Result<TcpStream, io::Error>) -> Option<Self> {
        let stream = stream.ok()?;
        
        stream.set_read_timeout(STREAM_TIMEOUT).ok()?;
        
        let mut reader = stream.take(REQUEST_SIZE_LIMIT);
        
        let mut headers = Vec::new();
        let mut body = Vec::new();
        
        // headers
        
        {
            
            let mut buffer = [0; CONNECTION_BUFFER_SIZE];
            
            loop {
                
                let bytes = reader.read(&mut buffer).ok()
                    .filter(|&bytes| bytes > 0)?;
                
                headers.extend_from_slice(&buffer[..bytes]);
                
                if let Some(position) = headers.windows(4).position(|curr| curr == b"\r\n\r\n") {
                    let index = position.checked_add(4)?;
                    body.append(&mut headers.split_off(index));
                    headers.truncate(index);
                    break;
                }
                
            }
            
        }
        
        // body
        
        {
            
            let content_length = chikuwa::tag_range(&headers, b"Content-Length: ", b"\r\n")
                .map(|range| &headers[range])
                .and_then(|value| str::from_utf8(value).ok())
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            
            if content_length > 0 {
                
                body.reserve(content_length);
                
                let mut buffer = [0; CONNECTION_BUFFER_SIZE];
                
                while body.len() < content_length {
                    
                    let bytes = reader.read(&mut buffer).ok()
                        .filter(|&bytes| bytes > 0)?;
                    
                    body.extend_from_slice(&buffer[..bytes]);
                    
                }
                
            }
            
            body.truncate(content_length);
            
        }
        
        let stream = Some(reader.into_inner());
        
        Some(Self {
            headers,
            body,
            stream,
        })
    }
    
    pub fn resource(&self) -> (&[u8], &[u8]) {
        let mut parts = self.headers.split(|&curr| curr == b' ');
        
        let method = parts.next().unwrap_or(&[]);
        let path = parts.next()
            .and_then(|path| path.split(|&curr| curr == b'?').next())
            .unwrap_or(&[]);
        
        (method, path)
    }
    
    pub fn param<'p, 'k: 'p>(&'p self, field: &'k [u8]) -> impl Iterator<Item = &'p str> {
        let range = chikuwa::tag_range(&self.headers, b"Content-Type: multipart/form-data; boundary=", b"\r\n");
        
        let payload = Payload {
            boundary: range.map_or(&[], |range| &self.headers[range]),
            content: &self.body,
        };
        
        payload.filter(move |(key, _)| key == &field)
            .filter_map(|(_, value)| str::from_utf8(value).ok())
    }
    
    pub fn start_response(&mut self, status: Status, content: ContentType) -> Result<Response, Box<dyn Error>> {
        Response::new(self.stream.take().ok_or("Response already sent")?, status, content)
    }
    
}

impl<'h, 'b> Iterator for Payload<'h, 'b> {
    
    type Item = (&'b [u8], &'b [u8]);
    
    fn next(&mut self) -> Option<Self::Item> {
        // windows method panics if given a zero as length
        if self.boundary.is_empty() {
            return None;
        }
        
        let range = chikuwa::tag_range(self.content, self.boundary, self.boundary)?;
        
        let mut current = &self.content[range.start..range.end];
        self.content = &self.content[range.end..];
        
        // key
        
        let range = chikuwa::tag_range(current, b"Content-Disposition: form-data; name=\"", b"\"\r\n")?;
        
        let key = &current[range.start..range.end];
        current = &current[range.end..][3..];
        
        // empty line
        
        current.get(..2).filter(|line| line == b"\r\n")?;
        current = &current[2..];
        
        // value
        
        current.get(current.len().saturating_sub(4)..).filter(|end| end == b"\r\n--")?;
        let value = &current[..current.len() - 4];
        
        Some((key, value))
    }
    
}

impl Response {
    
    fn new(stream: TcpStream, status: Status, content: ContentType) -> Result<Response, Box<dyn Error>> {
        stream.set_write_timeout(STREAM_TIMEOUT)?;
        
        let mut writer = BufWriter::new(stream);
        
        writer.write_all(b"HTTP/1.1 ")?;
        writer.write_all(status.as_bytes())?;
        writer.write_all(b"\r\n")?;
        
        writer.write_all(b"Content-Type: ")?;
        writer.write_all(content.as_bytes())?;
        writer.write_all(b"\r\n")?;
        
        writer.write_all(b"Cache-Control: ")?;
        writer.write_all(content.cache_policy())?;
        writer.write_all(b"\r\n")?;
        
        writer.write_all(b"Transfer-Encoding: chunked\r\n")?;
        writer.write_all(b"Connection: close\r\n")?;
        writer.write_all(b"\r\n")?;
        
        Ok(Self { writer })
    }
    
    pub fn send(&mut self, content: &[u8]) -> Result<(), Box<dyn Error>> {
        self.writer.write_all(format!("{:x}\r\n", content.len()).as_bytes())?;
        self.writer.write_all(content)?;
        self.writer.write_all(b"\r\n")?;
        Ok(())
    }
    
}

impl Drop for Response {
    
    fn drop(&mut self) {
        self.writer.write_all(b"0\r\n").ok();
        self.writer.write_all(b"\r\n").ok();
    }
    
}
