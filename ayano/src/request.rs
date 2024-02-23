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
        let mut body;
        
        // -------------------- headers --------------------
        
        loop {
            
            let bytes = reader.read(&mut buffer)
                .ok()
                .filter(|&bytes| bytes > 0)?;
            
            headers.extend_from_slice(&buffer[..bytes]);
            
            // separate body
            if let Some(position) = headers.windows(4).position(|curr| curr == b"\r\n\r\n") {
                body = headers.split_off(position.checked_add(4)?);
                break;
            }
            
        }
        
        // -------------------- body --------------------
        
        let content_length = chikuwa::subslice_range(&headers, b"Content-Length: ", b"\r\n")
            .map(|range| &headers[range])
            .and_then(|value| str::from_utf8(value).ok())
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0);
        
        body.reserve(content_length);
        
        while body.len() < content_length {
            
            let bytes = reader.read(&mut buffer)
                .ok()
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
    
    pub fn resource(&self) -> Option<(&[u8], &[u8])> {
        let mut parts = self.headers.split(|&curr| curr == b' ');
        
        let method = parts.next()?;
        let path = parts.next().and_then(|path| path.split(|&curr| curr == b'?').next())?;
        
        Some((method, path))
    }
    
    pub fn param<'p, 'k: 'p>(&'p self, field: &'k [u8]) -> impl Iterator<Item = &'p [u8]> {
        let range = chikuwa::subslice_range(&self.headers, b"Content-Type: multipart/form-data; boundary=", b"\r\n");
        
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
        while let Some(range) = chikuwa::subslice_range(self.content, self.boundary, self.boundary) {
            
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
    let range = chikuwa::subslice_range(data, b"Content-Disposition: form-data; name=\"", b"\"\r\n\r\n")?;
    
    let key = &data[range.start..range.end];
    let value = data[range.end..][5..].strip_suffix(b"\r\n--")?;
    
    Some((key, value))
}
