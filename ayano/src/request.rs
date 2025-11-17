use std::{
    io::{ self, Read, Error, ErrorKind },
    net::TcpStream,
};

use super::{
    REQUEST_SIZE_LIMIT, CONNECTION_BUFFER_SIZE, STREAM_TIMEOUT,
    StatusCode, ContentType, CacheControl,
    Headers, Body, QueryString, FormParams, Response,
};

pub struct Request {
    headers: Headers,
    body: Body,
    stream: Option<TcpStream>,
}

impl Request {
    
    pub(crate) fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_read_timeout(STREAM_TIMEOUT)?;
        
        let mut reader = chikuwa::LimitedReader::new(stream, REQUEST_SIZE_LIMIT)?;
        let (headers, body) = extract_headers_and_body(&mut reader)?;
        let stream = Some(reader.into_inner());
        
        Ok(Self {
            headers,
            body,
            stream,
        })
    }
    
    pub fn endpoint(&self) -> Option<&[u8]> {
        self.headers.endpoint()
    }
    
    pub fn get_header(&self, key: &[u8]) -> Option<&[u8]> {
        self.headers.get(key)
    }
    
    pub fn body_len(&self) -> usize {
        self.body.len()
    }
    
    pub fn query_string<'r, 'k>(&'r self, key: &'k [u8]) -> QueryString<'r, 'k> {
        self.headers.query_string(key)
    }
    
    pub fn form_params<'r: 'h, 'h, 'k>(&'r self, key: &'k [u8]) -> FormParams<'r, 'h, 'k> {
        self.body.form_params(&self.headers, key)
    }
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response> {
        let stream = self.stream.take()
            .ok_or(Error::other("Response already sent"))?;
        
        Response::new(stream, status, content, cache)
    }
    
}

fn extract_headers_and_body(reader: &mut impl Read) -> io::Result<(Headers, Body)> {
    let mut buffer = [0; CONNECTION_BUFFER_SIZE];
    let mut search_start_index = 0;
    
    let mut headers_content = Vec::new();
    
    #[allow(unused_assignments)]
    let mut body_content = Vec::new();
    
    // -------------------- headers --------------------
    
    loop {
        
        let bytes = match reader.read(&mut buffer) {
            Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed while reading headers")),
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        
        headers_content.extend_from_slice(&buffer[..bytes]);
        
        let headers_end = headers_content[search_start_index..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n");
        
        if let Some(headers_end) = headers_end {
            body_content = headers_content.split_off(search_start_index + headers_end + 4);
            break;
        }
        
        search_start_index = headers_content.len().saturating_sub(3);
        
    };
    
    let headers = Headers::new(headers_content);
    
    // -------------------- body --------------------
    
    // TODO: replace with usize::from_ascii in the future (https://github.com/rust-lang/rust/issues/134821)
    // body will be empty unless the request specifies a content length
    let content_length = headers.get(b"Content-Length")
        .and_then(|value| str::from_utf8(value).ok())
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    
    body_content.reserve_exact(content_length);
    
    while body_content.len() < content_length {
        
        let bytes = match reader.read(&mut buffer) {
            Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed while reading body")),
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        
        body_content.extend_from_slice(&buffer[..bytes]);
        
    }
    
    body_content.truncate(content_length);
    
    let body = Body::new(body_content);
    
    Ok((headers, body))
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod extract_headers_and_body {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.get(b"Host"), Some(b"placeholder".as_slice()));
            assert!(body.len() == 0);
        }
        
        #[test]
        fn headers_complex() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept: */*\r\n");
            content.extend_from_slice(b"Accept-Language: en-US,en;q=0.5\r\n");
            content.extend_from_slice(b"Accept-Encoding: gzip, deflate, br, zstd\r\n");
            content.extend_from_slice(b"Referer: http://placeholder/test\r\n");
            content.extend_from_slice(b"DNT: 1\r\n");
            content.extend_from_slice(b"Sec-GPC: 1\r\n");
            content.extend_from_slice(b"Connection: keep-alive\r\n");
            content.extend_from_slice(b"Sec-Fetch-Dest: empty\r\n");
            content.extend_from_slice(b"Sec-Fetch-Mode: cors\r\n");
            content.extend_from_slice(b"Sec-Fetch-Site: same-origin\r\n");
            content.extend_from_slice(b"Priority: u=0\r\n");
            content.extend_from_slice(b"Pragma: no-cache\r\n");
            content.extend_from_slice(b"Cache-Control: no-cache\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.get(b"Priority"), Some(b"u=0".as_slice()));
            assert!(body.len() == 0);
        }
        
        #[test]
        fn body_not_signaled() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn body_non_empty() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.get(b"Content-Length"), Some(b"4".as_slice()));
            assert!(body.len() == 4);
        }
        
        #[test]
        fn body_with_less_content_length() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 3\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.get(b"Content-Length"), Some(b"3".as_slice()));
            assert!(body.len() == 3);
        }
        
        #[test]
        fn body_with_more_content_length() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 5\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn body_with_excess_data() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"12345");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.get(b"Content-Length"), Some(b"4".as_slice()));
            assert!(body.len() == 4);
        }
        
        #[test]
        fn body_without_content_length() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert!(headers.get(b"Content-Length").is_none());
            assert!(body.len() == 0);
        }
        
        #[test]
        fn limit_exceeded() {
            // setup
            
            let mut content = Vec::with_capacity(REQUEST_SIZE_LIMIT as usize);
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            for _ in 0..REQUEST_SIZE_LIMIT {
                content.push(b'a');
            }
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.endpoint(), Some(b"GET /test/endpoint".as_slice()));
            assert!(body.len() == 0);
        }
        
    }
    
}
