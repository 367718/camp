use std::{
    io::{ self, Write },
    net::TcpStream,
};

use super::{
    STREAM_TIMEOUT, CONNECTION_BUFFER_SIZE,
    StatusCode, ContentType, CacheControl,
};

pub struct Response {
    buffer: Vec<u8>,
    stream: TcpStream,
}

impl Response {
    
    pub fn new(mut stream: TcpStream, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Self> {
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
