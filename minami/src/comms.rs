use std::{
    io::{ self, Read, Write },
    net::TcpStream,
    str,
    time::Duration,
};

const STREAM_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));
const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;
const REQUEST_SIZE_LIMIT: u64 = 512 * 1024 + 1;

#[derive(Copy, Clone)]
pub enum StatusCode {
    Ok,
    Error,
    NotFound,
}

#[derive(Copy, Clone)]
pub enum ContentType {
    Plain,
    Html,
    Icon,
    Css,
    Javascript,
}

#[derive(Copy, Clone)]
pub enum CacheControl {
    Static,
    Dynamic,
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
    buffer: Vec<u8>,
    stream: TcpStream,
}

impl StatusCode {
    
    fn as_header(self) -> &'static [u8] {
        match self {
            Self::Ok => b"HTTP/1.1 200 OK\r\n",
            Self::Error => b"HTTP/1.1 500 Internal Server Error\r\n",
            Self::NotFound => b"HTTP/1.1 404 Not Found\r\n",
        }
    }
    
}

impl ContentType {
    
    fn as_header(self) -> &'static [u8] {
        match self {
            Self::Plain => b"Content-Type: text/plain; charset=utf-8\r\n",
            Self::Html => b"Content-Type: text/html; charset=utf-8\r\n",
            Self::Icon => b"Content-Type: image/x-icon\r\n",
            Self::Css => b"Content-Type: text/css; charset=utf-8\r\n",
            Self::Javascript => b"Content-Type: text/javascript; charset=utf-8\r\n",
        }
    }
    
}

impl CacheControl {
    
    fn as_header(self) -> &'static [u8] {
        match self {
            Self::Static => b"Cache-Control: max-age=15552000, immutable\r\n",
            Self::Dynamic => b"Cache-Control: no-cache, no-store\r\n",
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
        
        // -------------------- headers --------------------
        
        {
            
            let mut buffer = [0; CONNECTION_BUFFER_SIZE];
            
            loop {
                
                let bytes = reader.read(&mut buffer)
                    .ok()
                    .filter(|&bytes| bytes > 0)?;
                
                headers.extend_from_slice(&buffer[..bytes]);
                
                // separate body
                if let Some(position) = headers.windows(4).position(|curr| curr == b"\r\n\r\n") {
                    let index = position.checked_add(4)?;
                    body.append(&mut headers.split_off(index));
                    headers.truncate(index);
                    break;
                }
                
            }
            
        }
        
        // -------------------- body --------------------
        
        {
            
            let content_length = chikuwa::tag_range(&headers, b"Content-Length: ", b"\r\n")
                .map(|range| &headers[range])
                .and_then(|value| str::from_utf8(value).ok())
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            
            if body.len() < content_length {
                
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
    
    pub fn param<'p, 'k: 'p>(&'p self, field: &'k [u8]) -> impl Iterator<Item = &'p [u8]> {
        let range = chikuwa::tag_range(&self.headers, b"Content-Type: multipart/form-data; boundary=", b"\r\n");
        
        let payload = Payload {
            boundary: range.map_or(&[], |range| &self.headers[range]),
            content: &self.body,
        };
        
        payload.filter(move |(key, _)| key == &field).map(|(_, value)| value)
    }
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response> {
        let stream = self.stream.take()
            .ok_or(io::Error::new(io::ErrorKind::Other, "Response already sent"))?;
        
        Response::new(stream, status, content, cache)
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
    
    fn new(mut stream: TcpStream, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Self> {
        let mut buffer = Vec::with_capacity(CONNECTION_BUFFER_SIZE);
        
        buffer.extend_from_slice(status.as_header());
        buffer.extend_from_slice(content.as_header());
        buffer.extend_from_slice(cache.as_header());
        
        buffer.extend_from_slice(b"Transfer-Encoding: chunked\r\n");
        buffer.extend_from_slice(b"Connection: close\r\n");
        buffer.extend_from_slice(b"\r\n");
        
        stream.set_write_timeout(STREAM_TIMEOUT)?;
        stream.write_all(&buffer)?;
        
        buffer.clear();
        
        Ok(Self {
            buffer,
            stream,
        })
    }
    
}

impl Write for Response {
    
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = buf.len().min(self.buffer.capacity() - self.buffer.len());
        
        self.buffer.extend_from_slice(&buf[..size]);
        
        if self.buffer.capacity() == self.buffer.len() {
            self.flush()?;
        }
        
        Ok(size)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        if ! self.buffer.is_empty() {
            
            write!(&mut self.stream, "{:x}\r\n", self.buffer.len())?;
            self.stream.write_all(&self.buffer)?;
            self.stream.write_all(b"\r\n")?;
            
            self.buffer.clear();
            
        }
        
        Ok(())
    }
    
}

impl Drop for Response {
    
    fn drop(&mut self) {
        self.flush().ok();
        self.stream.write_all(b"0\r\n\r\n").ok();
    }
    
}
