use std::io::{ self, Write };

use super::{
    CONNECTION_BUFFER_SIZE,
    StatusCode, ContentType, CacheControl,
};

pub struct Response<S: Write> {
    buffer: Vec<u8>,
    stream: S,
}

impl<S: Write> Response<S> {
    
    pub(crate) fn new(mut stream: S, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Self> {
        let mut buffer = Vec::with_capacity(CONNECTION_BUFFER_SIZE);
        
        buffer.extend_from_slice(status.into_header());
        buffer.extend_from_slice(content.into_header());
        buffer.extend_from_slice(cache.into_header());
        
        buffer.extend_from_slice(b"Transfer-Encoding: chunked\r\n");
        buffer.extend_from_slice(b"Connection: close\r\n");
        buffer.extend_from_slice(b"\r\n");
        
        stream.write_all(&buffer)?;
        buffer.clear();
        
        Ok(Self {
            buffer,
            stream,
        })
    }
    
}

impl<S: Write> Write for Response<S> {
    
    fn write(&mut self, content: &[u8]) -> io::Result<usize> {
        let size = content.len()
            .min(CONNECTION_BUFFER_SIZE.saturating_sub(self.buffer.len()));
        
        self.buffer.extend_from_slice(&content[..size]);
        
        if self.buffer.len() >= CONNECTION_BUFFER_SIZE {
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

impl<S: Write> Drop for Response<S> {
    
    fn drop(&mut self) {
        self.flush().ok();
        self.stream.write_all(b"0\r\n\r\n").ok();
    }
    
}
