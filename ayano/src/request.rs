use std::io::{ self, Read, Write, Error, ErrorKind };

use super::{
    REQUEST_SIZE_LIMIT, CONNECTION_BUFFER_SIZE,
    StatusCode, ContentType, CacheControl,
    Response,
};

pub struct Request<S: Read + Write> {
    headers: Vec<u8>,
    body: Vec<u8>,
    stream: Option<S>,
}

pub struct FormData<'r, 'k> {
    content: &'r [u8],
    boundary: &'r [u8],
    key: &'k [u8],
}

impl<S: Read + Write> Request<S> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(stream: S) -> io::Result<Self> {
        let mut reader = stream.take(REQUEST_SIZE_LIMIT);
        let mut buffer = [0; CONNECTION_BUFFER_SIZE];
        let mut search_start_index = 0;
        
        let mut headers = Vec::new();
        let mut body;
        
        // -------------------- headers --------------------
        
        loop {
            
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
                // consider "\r\n\r\n" as part of header
                body = headers.split_off(search_start_index + headers_end + 4);
                break;
            }
            
            // previous read might have included "\r\n\r"
            search_start_index = headers.len().saturating_sub(3);
            
        }
        
        // -------------------- body --------------------
        
        // TODO: replace with usize::from_ascii in the future (https://github.com/rust-lang/rust/issues/134821)
        // body will be empty unless the request specifies a content length
        let content_length = chikuwa::delimited_range(&headers, b"Content-Length:", b"\r\n")
            .and_then(|range| str::from_utf8(&headers[range]).ok())
            .and_then(|value| value.trim().parse::<usize>().ok())
            .unwrap_or(0);
        
        if body.len() < content_length {
            
            // limit reservation since content_length value is untrusted
            let remaining = (content_length - body.len())
                .min(usize::try_from(reader.limit()).expect("Unsupported platform"));
            
            body.reserve_exact(remaining);
            reader.set_limit(remaining as u64);
            
            reader.read_to_end(&mut body)?;
            
            if body.len() < content_length && reader.limit() > 0 {
                return Err(Error::new(ErrorKind::UnexpectedEof, "Connection closed while reading body"));
            }
            
        }
        
        body.truncate(content_length);
        
        // -------------------- stream --------------------
        
        let stream = Some(reader.into_inner());
        
        // -------------------- response --------------------
        
        Ok(Self {
            headers,
            body,
            stream,
        })
        
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn endpoint(&self) -> Option<&[u8]> {
        // GET /test/resource?fkey=fvalue HTTP/1.1\r\n
        
        // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
        // to the left of '\r' => GET /test/resource?fkey=fvalue HTTP/1.1
        let first_line = self.headers.splitn(2, |&curr| curr == b'\r').next()?;
        
        // TODO: replace with slice::rsplit_once in the future (https://github.com/rust-lang/rust/issues/112811)
        // to the left of ' ' => GET /test/resource?fkey=fvalue
        let endpoint = first_line.rsplitn(2, |&curr| curr == b' ').nth(1)?;
        
        // strip '?', if any => GET /test/resource
        endpoint.split(|&curr| curr == b'?').next()
    }
    
    pub fn get_header(&self, key: &[u8]) -> Option<&[u8]> {
        // Host: placeholder\r\n
        
        // : placeholder
        chikuwa::delimited_range(&self.headers, key, b"\r\n")
            //  placeholder
            .and_then(|range| self.headers[range].strip_prefix(b":"))
            // placeholder
            .map(<[u8]>::trim_ascii_start)
    }
    
    pub fn form_data<'r, 'k>(&'r self, key: &'k [u8]) -> FormData<'r, 'k> {
        // Content-Type: multipart/form-data; charset=utf-8; boundary=9999999999999999999999999999
        
        let boundary = self.get_header(b"Content-Type")
            .and_then(|value| {
                
                // parameters are key-value pairs separated by ';' and preceded by mime type
                let mut parts = value.split(|&curr| curr == b';').map(<[u8]>::trim_ascii);
                
                // mime type must be "multipart/form-data" in this case
                if ! parts.next()?.eq_ignore_ascii_case(b"multipart/form-data") {
                    return None;
                }
                
                // any number of other parameters may be present before and after "boundary"
                let boundary = parts.find_map(|part| {
                    
                    let mut pair = part.splitn(2, |&curr| curr == b'=').map(<[u8]>::trim_ascii);
                    
                    let pair_key = pair.next()?;
                    let pair_value = pair.next()?;
                    
                    if pair_key.eq_ignore_ascii_case(b"boundary") {
                        Some(pair_value)
                    } else {
                        None
                    }
                    
                })?;
                
                // strip optional surrounding quotes
                let unquoted = boundary.strip_prefix(b"\"")
                    .and_then(|value| value.strip_suffix(b"\""))
                    .unwrap_or(boundary);
                
                Some(unquoted)
                
            })
            .unwrap_or(&[]);
        
        FormData {
            content: &self.body,
            boundary,
            key,
        }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response<S>> {
        let stream = self.stream.take()
            .ok_or(Error::other("Response already sent"))?;
        
        Response::new(stream, status, content, cache)
    }
    
}

impl<'r> Iterator for FormData<'r, '_> {
    
    type Item = &'r [u8];
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // --9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #1"\r\n
        // \r\n
        // placeholder value #1\r\n
        // --9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #2"\r\n
        // \r\n
        // placeholder value #2\r\n
        // --9999999999999999999999999999--\r\n
        
        while let Some(range) = chikuwa::delimited_range(self.content, self.boundary, self.boundary) {
            
            // Content-Disposition: form-data; name="placeholder key"\r\n
            // \r\n
            // placeholder value\r\n
            // --
            
            let current = &self.content[range];
            self.content = &self.content[range.end..];
            
            let data = chikuwa::delimited_range(current, b"name=\"", b"\"\r\n\r\n")?;
            
            let key = &current[data];
            let value = &current[data.end + 5..].strip_suffix(b"\r\n--")?;
            
            if key.is_empty() || value.is_empty() {
                continue;
            }
            
            if key.eq_ignore_ascii_case(self.key) {
                return Some(value);
            }
            
        }
        
        None
        
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod new {
        
        use super::*;
        
        #[test]
        fn simple() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            let control = request.headers
                .into_iter()
                .chain(request.body)
                .collect::<Vec<u8>>();
            
            assert_eq!(content, control);
        }
        
        #[test]
        fn headers_complex() {
            use std::io::Cursor;
            
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
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            let control = request.headers
                .into_iter()
                .chain(request.body)
                .collect::<Vec<u8>>();
            
            assert_eq!(content, control);
        }
        
        #[test]
        fn body_not_signaled() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn body_non_empty() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            let control = request.headers
                .into_iter()
                .chain(request.body)
                .collect::<Vec<u8>>();
            
            assert_eq!(content, control);
        }
        
        #[test]
        fn case_mixed_body_length() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-LENGTH:4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            let control = request.headers
                .into_iter()
                .chain(request.body)
                .collect::<Vec<u8>>();
            
            assert_eq!(content, control);
        }
        
        #[test]
        fn body_with_less_content_length() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 3\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            assert_eq!(b"123".as_slice(), request.body);
        }
        
        #[test]
        fn body_with_more_content_length() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 5\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn body_with_excess_data() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 4\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"12345");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            assert_eq!(b"1234".as_slice(), request.body);
        }
        
        #[test]
        fn body_without_content_length() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"1234");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            assert!(request.body.is_empty());
        }
        
        #[test]
        fn limit_exceeded_by_headers() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::with_capacity(REQUEST_SIZE_LIMIT as usize);
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            
            for _ in 0..REQUEST_SIZE_LIMIT {
                content.push(b'a');
            }
            
            content.extend_from_slice(b"\r\n\r\n");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            assert!(output.is_err());
        }
        
        #[test]
        fn limit_exceeded_by_body() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::with_capacity(REQUEST_SIZE_LIMIT as usize);
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            write!(content, "Content-Length: {}\r\n", REQUEST_SIZE_LIMIT + 1).ok();
            content.extend_from_slice(b"\r\n");
            
            for _ in 0..REQUEST_SIZE_LIMIT {
                content.push(b'a');
            }
            
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = Request::new(Cursor::new(content.clone()));
            
            // control
            
            let request = output.unwrap();
            
            assert_eq!(REQUEST_SIZE_LIMIT as usize, request.headers.len() + request.body.len());
        }
        
    }
    
    #[cfg(test)]
    mod endpoint {
        
        use super::*;
        
        #[test]
        fn simple() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET /test/resource".as_slice()));
        }
        
        #[test]
        fn with_query_string() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?fkey=fvalue HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET /test/resource".as_slice()));
        }
        
        #[test]
        fn extra_whitespace() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource  HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET /test/resource ".as_slice()));
        }
        
        #[test]
        fn without_method() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"/test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"/test/resource".as_slice()));
        }
        
        #[test]
        fn without_path() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET".as_slice()));
        }
        
        #[test]
        fn method_only() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
    #[cfg(test)]
    mod get_header {
        
        use super::*;
        
        #[test]
        fn simple() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"host";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn complex() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
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
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Sec-GPC";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"1");
        }
        
        #[test]
        fn duplicate() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Host: non-existant\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"host";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn case_mixed_content() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"hOsT: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"host";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn case_mixed_key() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"hOSt";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn no_whitespace() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:*/*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Accept";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"*/*");
        }
        
        #[test]
        fn extra_whitespace() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:    */*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Accept";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"*/*");
        }
        
        #[test]
        fn empty() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Accept";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"");
        }
        
        #[test]
        fn non_existant() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept: */*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Content-Length";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
    #[cfg(test)]
    mod form_data {
        
        use super::*;
        
        #[test]
        fn simple() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn complex() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 203\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"second";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn duplicate() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 202\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_name() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 105\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data;\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixed_name() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"CONTENT-DISPOSITION:form-data;naMe=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"inPUT";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixed_key() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"inPUT";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn nonexistent_key() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 203\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"third";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_name() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 198\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"first";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_value() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 201\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"first";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_pairs() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 0\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_whitespace_in_boundary() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary = 9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn mimetype_last() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: boundary=9999999999999999999999999999; multipart/form-data\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_parameter_before_boundary() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; charset=utf-8; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_parameter_after_boundary() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999; charset=utf-8\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixed_boundary() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; BOUNDARY=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_boundary() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn wrong_boundary() {
            use std::io::Cursor;
            
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999998\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
}
