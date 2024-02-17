use std::{
    io::{ self, Read },
    net::TcpStream,
    str,
};

use super::{
    STREAM_TIMEOUT, REQUEST_SIZE_LIMIT, CONNECTION_BUFFER_SIZE,
    StatusCode, ContentType, CacheControl, Response,
};

pub struct Request {
    headers: Vec<u8>,
    body: Vec<u8>,
    stream: Option<TcpStream>,
}

struct Params<'h, 'b> {
    boundary: &'h [u8],
    content: &'b [u8],
}

impl Request {
    
    pub(crate) fn new(stream: TcpStream) -> Option<Self> {
        stream.set_read_timeout(STREAM_TIMEOUT).ok()?;
        
        let mut reader = stream.take(REQUEST_SIZE_LIMIT);
        let mut buffer = [0; CONNECTION_BUFFER_SIZE];
        
        let mut headers = Vec::new();
        let mut body = Vec::new();
        
        // -------------------- headers --------------------
        
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
        
        // -------------------- body --------------------
        
        let content_length = chikuwa::tag_range(&headers, b"Content-Length: ", b"\r\n")
            .map(|range| &headers[range])
            .and_then(|value| str::from_utf8(value).ok())
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        
        body.reserve(content_length);
        
        while body.len() < content_length {
            
            let bytes = reader.read(&mut buffer).ok()
                .filter(|&bytes| bytes > 0)?;
            
            body.extend_from_slice(&buffer[..bytes]);
            
        }
        
        body.truncate(content_length);
        
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
        
        let payload = Params {
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

impl<'h, 'b> Iterator for Params<'h, 'b> {
    
    type Item = (&'b [u8], &'b [u8]);
    
    fn next(&mut self) -> Option<Self::Item> {
        // windows method panics if given a zero as length
        if self.boundary.is_empty() {
            return None;
        }
        
        while let Some(range) = chikuwa::tag_range(self.content, self.boundary, self.boundary) {
            
            let item = build_pair(&self.content[range.start..range.end]);
            self.content = &self.content[range.end..];
            
            if item.is_some() {
                return item;
            }
            
        }
        
        None
    }
    
}

fn build_pair(data: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut current = data;
    
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
