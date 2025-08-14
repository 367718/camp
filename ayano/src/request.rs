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
    
    pub(crate) fn new(mut stream: TcpStream) -> io::Result<Self> {
        stream.set_read_timeout(STREAM_TIMEOUT)?;
        
        let (headers, body) = extract_headers_and_body(&mut stream)?;
        
        Ok(Self {
            headers,
            body,
            stream: Some(stream),
        })
    }
    
    pub fn method_and_path(&self) -> Option<(&[u8], &[u8])> {
        extract_method_and_path(&self.headers)
    }
    
    pub fn form_data(&self) -> Option<FormData<'_>> {
        FormData::new(&self.headers, &self.body)
    }
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response> {
        let stream = self.stream.take()
            .ok_or(Error::other("Response already sent"))?;
        
        Response::new(stream, status, content, cache)
    }
    
}

fn extract_headers_and_body(reader: &mut impl Read) -> io::Result<(Vec<u8>, Vec<u8>)> {
    let mut buffer = [0; CONNECTION_BUFFER_SIZE];
    let mut search_start_index = 0;
    
    let mut headers = Vec::new();
    let mut body = Vec::new();
    
    // -------------------- headers --------------------
    
    while headers.len() < REQUEST_SIZE_LIMIT {
        
        let bytes = match reader.read(&mut buffer) {
            Ok(0) => return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed while reading headers")),
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        
        headers.extend_from_slice(&buffer[..bytes]);
        
        let headers_end = headers[search_start_index..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n");
        
        if let Some(headers_end) = headers_end {
            body = headers.split_off(search_start_index + headers_end + 4);
            break;
        }
        
        search_start_index = headers.len().saturating_sub(3);
        
    };
    
    headers.truncate(REQUEST_SIZE_LIMIT);
    
    let body_limit = REQUEST_SIZE_LIMIT.saturating_sub(headers.len());
    
    // -------------------- body --------------------
    
    // body will be empty unless the request specifies a content length
    let content_length = chikuwa::subslice_range(&headers, b"Content-Length:", b"\r\n")
        .map(|range| &headers[range])
        .and_then(|value| str::from_utf8(value).ok())
        .and_then(|value| value.trim().parse::<usize>().ok())
        .map_or(0, |value| value.min(body_limit));
    
    if body.len() < content_length {
        
        body.reserve_exact(content_length);
        
        let remaining = content_length - body.len();
        let result = reader.take(remaining as u64)
            .read_to_end(&mut body);
        
        if let Err(error) = result && error.kind() != ErrorKind::UnexpectedEof {
            return Err(error);
        }
        
    }
    
    body.truncate(content_length);
    
    Ok((headers, body))
}

fn extract_method_and_path(data: &[u8]) -> Option<(&[u8], &[u8])> {
    let mut parts = data.split(|&curr| curr == b' ');
    
    let method = parts.next()?;
    
    // strip query component
    let path = parts.next()
        .and_then(|path| path.split(|&curr| curr == b'?').next())?;
    
    Some((method, path))
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
            
            assert_eq!(headers, content);
            assert!(body.is_empty());
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
            
            assert_eq!(headers, content);
            assert!(body.is_empty());
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
            
            assert_eq!(headers, content[..content.len() - body.len()]);
            assert_eq!(body, b"1234");
        }
        
        #[test]
        fn body_without_space_in_content_length() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length:4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers, content[..content.len() - body.len()]);
            assert_eq!(body, b"1234");
        }
        
        #[test]
        fn body_with_multiple_spaces_in_content_length() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length:   4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers, content[..content.len() - body.len()]);
            assert_eq!(body, b"1234");
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
            
            assert_eq!(headers, content[..content.len() - body.len() - 1]);
            assert_eq!(body, b"123");
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
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers, content[..content.len() - body.len()]);
            assert_eq!(body, b"1234");
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
            
            assert_eq!(headers, content[..content.len() - body.len() - 1]);
            assert_eq!(body, b"1234");
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
            
            assert_eq!(headers, content[..content.len() - 4]);
            assert!(body.is_empty());
        }
        
        #[test]
        fn limit_exceeded() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            
            for _ in 0..REQUEST_SIZE_LIMIT {
                content.push(b'a');
            }
            
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = extract_headers_and_body(&mut &content[..]);
            
            // control
            
            assert!(output.is_ok());
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(headers.len() + body.len(), REQUEST_SIZE_LIMIT as usize);
        }
        
    }
    
    #[cfg(test)]
    mod extract_method_and_path {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = extract_method_and_path(&mut &content[..]);
            
            // control
            
            assert!(output.is_some());
            
            let (method, path) = output.unwrap();
            
            assert_eq!(method, b"GET");
            assert_eq!(path, b"/test/endpoint");
        }
        
        #[test]
        fn no_path() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET HTTP/1.1\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = extract_method_and_path(&mut &content[..]);
            
            // control
            
            assert!(output.is_some());
            
            let (method, path) = output.unwrap();
            
            assert_eq!(method, b"GET");
            assert_eq!(path, b"HTTP/1.1\r\n\r\n");
        }
        
        #[test]
        fn malformed() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"HTTP/1.1\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = extract_method_and_path(&mut &content[..]);
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
}
