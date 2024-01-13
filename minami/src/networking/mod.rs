mod request;
mod response;

use std::time::Duration;

pub use request::Request;
pub use response::Response;

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
