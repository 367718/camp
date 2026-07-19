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

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn empty() {
        // setup
        
        let mut content = Vec::new();
        
        let mut control = Vec::new();
        control.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
        control.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
        control.extend_from_slice(b"Cache-Control: no-cache, no-store\r\n");
        control.extend_from_slice(b"Transfer-Encoding: chunked\r\n");
        control.extend_from_slice(b"Connection: close\r\n");
        control.extend_from_slice(b"\r\n");
        control.extend_from_slice(b"0\r\n\r\n");
        
        // operation
        
        let output = Response::new(&mut content, StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic);
        
        // control
        
        assert!(output.is_ok());
        
        drop(output);
        
        assert_eq!(content, control);
    }
    
    #[test]
    fn short() {
        // setup
        
        let mut content = Vec::new();
        
        let mut control = Vec::new();
        control.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
        control.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
        control.extend_from_slice(b"Cache-Control: no-cache, no-store\r\n");
        control.extend_from_slice(b"Transfer-Encoding: chunked\r\n");
        control.extend_from_slice(b"Connection: close\r\n");
        control.extend_from_slice(b"\r\n");
        control.extend_from_slice(b"6\r\n");
        control.extend_from_slice(b"qwerty\r\n");
        control.extend_from_slice(b"0\r\n\r\n");
        
        // operation
        
        let output = Response::new(&mut content, StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic);
        
        let mut output = output.unwrap();
        
        output.write_all(b"qwerty").unwrap();
        
        // control
        
        drop(output);
        
        assert_eq!(content, control);
    }
    
    #[test]
    fn long() {
        // setup
        
        let mut content = Vec::new();
        
        let mut control = Vec::new();
        control.extend_from_slice(b"HTTP/1.1 200 OK\r\n");
        control.extend_from_slice(b"Content-Type: text/plain; charset=utf-8\r\n");
        control.extend_from_slice(b"Cache-Control: no-cache, no-store\r\n");
        control.extend_from_slice(b"Transfer-Encoding: chunked\r\n");
        control.extend_from_slice(b"Connection: close\r\n");
        control.extend_from_slice(b"\r\n");
        write!(control, "{:x}\r\n", CONNECTION_BUFFER_SIZE).ok();
        
        for _ in 0..CONNECTION_BUFFER_SIZE {
            control.push(b'a');
        }
        
        control.extend_from_slice(b"\r\n");
        control.extend_from_slice(b"6\r\n");
        control.extend_from_slice(b"qwerty\r\n");
        control.extend_from_slice(b"0\r\n\r\n");
        
        // operation
        
        let output = Response::new(&mut content, StatusCode::Ok, ContentType::Plain, CacheControl::Dynamic);
        
        let mut output = output.unwrap();
        
        for _ in 0..CONNECTION_BUFFER_SIZE {
            output.write_all(b"a").unwrap();
        }
        
        output.write_all(b"qwerty").unwrap();
        
        // control
        
        drop(output);
        
        assert_eq!(content, control);
    }
    
}
