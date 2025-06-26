use std::{
    io::{ self, Read, Error, ErrorKind },
    net::TcpStream,
};

use super::{
    REQUEST_SIZE_LIMIT, CONNECTION_BUFFER_SIZE, STREAM_TIMEOUT,
    StatusCode, ContentType, CacheControl, FormData, Response,
};

pub struct Request {
    headers: Vec<u8>,
    body: Vec<u8>,
    stream: Option<TcpStream>,
}

impl Request {
    
    pub(crate) fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_read_timeout(STREAM_TIMEOUT)?;
        
        let mut reader = stream.take(REQUEST_SIZE_LIMIT);
        let mut buffer = [0; CONNECTION_BUFFER_SIZE];
        
        let mut headers = Vec::new();
        let mut body;
        
        // -------------------- headers --------------------
        
        loop {
            
            let bytes = reader.read(&mut buffer)?;
            
            if bytes == 0 {
                return Err(Error::from(ErrorKind::Interrupted));
            }
            
            headers.extend_from_slice(&buffer[..bytes]);
            
            // separate body
            if let Some(position) = headers.windows(4).position(|curr| curr == b"\r\n\r\n") {
                body = headers.split_off(position + 4);
                break;
            }
            
        }
        
        // -------------------- body --------------------
        
        // body will be empty unless the request specifies a content length
        let content_length = chikuwa::subslice_range(&headers, b"Content-Length: ", b"\r\n")
            .map(|range| &headers[range])
            .and_then(|value| str::from_utf8(value).ok())
            .and_then(|value| value.parse::<u64>().ok())
            .and_then(|value| usize::try_from(value.min(reader.limit())).ok())
            .unwrap_or(0);
        
        body.reserve_exact(content_length);
        
        while body.len() < content_length {
            
            let bytes = reader.read(&mut buffer)?;
            
            if bytes == 0 {
                return Err(Error::from(ErrorKind::Interrupted));
            }
            
            body.extend_from_slice(&buffer[..bytes]);
            
        }
        
        body.truncate(content_length);
        
        let stream = Some(reader.into_inner());
        
        Ok(Self {
            headers,
            body,
            stream,
        })
    }
    
    pub fn method_and_path(&self) -> Option<(&[u8], &[u8])> {
        let mut parts = self.headers.split(|&curr| curr == b' ');
        
        let method = parts.next()?;
        
        // strip query component
        let path = parts.next()
            .and_then(|path| path.split(|&curr| curr == b'?').next())?;
        
        Some((method, path))
    }
    
    pub fn form_data(&self) -> Option<FormData> {
        let range = chikuwa::subslice_range(&self.headers, b"Content-Type: multipart/form-data; boundary=", b"\r\n")?;
        
        let boundary = &self.headers[range];
        let content = &self.body;
        
        Some(FormData::new(boundary, content))
    }
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response> {
        let stream = self.stream.take()
            .ok_or(Error::other("Response already sent"))?;
        
        Response::new(stream, status, content, cache)
    }
    
}
