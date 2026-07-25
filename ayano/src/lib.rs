mod request;
mod response;

use std::{
    io,
    net::{ TcpListener, TcpStream },
    time::Duration,
};

use request::Request;
use response::Response;

pub use request::FormData;

pub type ServerRequest = Request<TcpStream>;
pub type ServerResponse = Response<TcpStream>;

const CONNECTION_BUFFER_SIZE: usize = 8 * 1024;
const REQUEST_SIZE_LIMIT: u64 = 64 * 1024;

pub enum StatusCode {
    Ok,
    Error,
    NotFound,
}

pub enum ContentType {
    Plain,
    Html,
    Css,
    Javascript,
}

pub enum CacheControl {
    Static,
    Dynamic,
}

pub struct Server {
    listener: TcpListener,
    read_timeout: Option<Duration>,
    write_timeout: Option<Duration>,
}

impl StatusCode {
    
    const fn into_header(self) -> &'static [u8] {
        match self {
            Self::Ok => b"HTTP/1.1 200 OK\r\n",
            Self::Error => b"HTTP/1.1 500 Internal Server Error\r\n",
            Self::NotFound => b"HTTP/1.1 404 Not Found\r\n",
        }
    }
    
}

impl ContentType {
    
    const fn into_header(self) -> &'static [u8] {
        match self {
            Self::Plain => b"Content-Type: text/plain; charset=utf-8\r\n",
            Self::Html => b"Content-Type: text/html; charset=utf-8\r\n",
            Self::Css => b"Content-Type: text/css; charset=utf-8\r\n",
            Self::Javascript => b"Content-Type: text/javascript; charset=utf-8\r\n",
        }
    }
    
}

impl CacheControl {
    
    const fn into_header(self) -> &'static [u8] {
        match self {
            Self::Static => b"Cache-Control: max-age=15552000, immutable\r\n",
            Self::Dynamic => b"Cache-Control: no-cache, no-store\r\n",
        }
    }
    
}

impl Server {
    
    pub fn bind(address: &str, read_timeout: u64, write_timeout: u64) -> io::Result<Self> {
        Ok(Self {
            listener: TcpListener::bind(address)?,
            read_timeout: Some(Duration::from_secs(read_timeout)),
            write_timeout: Some(Duration::from_secs(write_timeout)),
        })
    }
    
    pub fn accept(&mut self) -> io::Result<Request<TcpStream>> {
        let (stream, _) = self.listener.accept()?;
        
        stream.set_nodelay(true)?;
        stream.set_read_timeout(self.read_timeout)?;
        stream.set_write_timeout(self.write_timeout)?;
        
        Request::new(stream)
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn request_and_response() {
        use std::io::{ Cursor, Write };
        
        // setup
        
        let mut content = Vec::new();
        content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
        content.extend_from_slice(b"Host: placeholder\r\n");
        content.extend_from_slice(b"Content-Length: 4\r\n");
        content.extend_from_slice(b"\r\n");
        content.extend_from_slice(b"1234");
        
        let mut control = Vec::new();
        control.extend_from_slice(b"HTTP/1.1 500 Internal Server Error\r\n");
        control.extend_from_slice(b"Content-Type: text/html; charset=utf-8\r\n");
        control.extend_from_slice(b"Cache-Control: max-age=15552000, immutable\r\n");
        control.extend_from_slice(b"Transfer-Encoding: chunked\r\n");
        control.extend_from_slice(b"Connection: close\r\n");
        control.extend_from_slice(b"\r\n");
        control.extend_from_slice(b"7\r\n");
        control.extend_from_slice(b"qwerty!\r\n");
        control.extend_from_slice(b"0\r\n\r\n");
        
        // operation
        
        let output = Request::new(Cursor::new(&mut content));
        
        let mut request = output.unwrap();
        let mut response = request.start_response(StatusCode::Error, ContentType::Html, CacheControl::Static).unwrap();
        
        // control
        
        assert_eq!(request.endpoint(), Some((b"GET".as_slice(), b"/test/endpoint".as_slice())));
        
        response.write_all(b"qwerty").unwrap();
        response.write_all(b"!").unwrap();
        
        drop(response);
        
        assert_eq!(content[73..], control);
        
    }
    
}
