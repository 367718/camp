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

pub struct FormData<'r, 'p> {
    content: &'r [u8],
    boundary: &'r [u8],
    param: &'p [u8],
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
    
    
    pub fn endpoint(&self) -> Option<(&[u8], &[u8])> {
        // GET /test/resource?fkey=fvalue HTTP/1.1\r\n
        
        // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
        let first_line = self.headers.splitn(2, |&byte| byte == b'\r').next().unwrap();
        
        let mut components = first_line.split(|&byte| byte == b' ')
            .filter(|component| ! component.is_empty());
        
        // request method
        let method = components.next()?;
        
        // request target
        let target = components.next()?
            // strip query string
            .split(|&byte| byte == b'?').next().unwrap();
        
        Some((method, target))
    }
    
    pub fn header(&self, key: &[u8]) -> Option<&[u8]> {
        // Host: placeholder\r\n
        
        // non-compliant with rfc 9110:
        // - whitespace is allowed in key
        // - only "U+0020 SPACE" and "U+0009 HORIZONTAL TAB" should be considered as whitespace
        
        chikuwa::delimited_range(&self.headers, key, b"\r\n")
            .map(|range| &self.headers[range])
            .and_then(|value| value.strip_prefix(b":"))
            .map(<[u8]>::trim_ascii)
    }
    
    pub fn form_data<'r, 'p>(&'r self, param: &'p [u8]) -> FormData<'r, 'p> {
        // Content-Type: multipart/form-data; charset=utf-8; boundary=9999999999999999999999999999
        
        // non-compliant with rfc 9110:
        // - media-type should precede parameter list
        // - only "U+0020 SPACE" and "U+0009 HORIZONTAL TAB" should be considered as whitespace
        
        let boundary = self.header(b"Content-Type")
            .and_then(|parameters| {
                
                let value = parameters
                    .split(|&byte| byte == b';')
                    .filter_map(|parameter| {
                        
                        // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
                        let mut pair = parameter.splitn(2, |&byte| byte == b'=')
                            .map(<[u8]>::trim_ascii);
                        
                        let key = pair.next().unwrap();
                        let value = pair.next()?;
                        
                        Some((key, value))
                        
                    })
                    .find_map(|(key, value)| {
                        
                        if key.eq_ignore_ascii_case(b"boundary") {
                            Some(value)
                        } else {
                            None
                        }
                        
                    })?;
                
                // strip optional surrounding quotes
                let unquoted = match value {
                    [b'"', inner @ .., b'"'] => inner,
                    _ => value,
                };
                
                Some(unquoted)
                
            })
            .unwrap_or(&[]);
        
        FormData {
            content: &self.body,
            boundary,
            param,
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
        // Content-Disposition: form-data; name="placeholder key #1" filename="example.txt"\r\n
        // \r\n
        // placeholder value #1\r\n
        // --9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #2" filename="example.txt"\r\n
        // \r\n
        // placeholder value #2\r\n
        // --9999999999999999999999999999--
        
        while let Some(range) = chikuwa::delimited_range(self.content, self.boundary, self.boundary) {
            
            // \r\n
            // Content-Disposition: form-data; name="placeholder key" filename="example.txt"\r\n
            // \r\n
            // placeholder value\r\n
            // --
            
            let current = &self.content[range];
            self.content = &self.content[range.end..];
            
            // -------------------- parameter --------------------
            
            let Some(parameters) = chikuwa::delimited_range(current, b"Content-Disposition:", b"\r\n\r\n") else {
                continue;
            };
            
            let relevant = current[parameters]
                .split(|&byte| byte == b';')
                .filter_map(|parameter| {
                    
                    // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
                    let mut pair = parameter.splitn(2, |&byte| byte == b'=')
                        .map(<[u8]>::trim_ascii);
                    
                    let key = pair.next().unwrap();
                    let value = pair.next()?;
                    
                    Some((key, value))
                    
                })
                .filter(|(key, _)| key.eq_ignore_ascii_case(b"name"))
                .any(|(_, value)| matches!(value, [b'"', inner @ .., b'"'] if inner.eq_ignore_ascii_case(self.param)));
            
            if ! relevant {
                continue;
            }
            
            // -------------------- payload --------------------
            
            if ! current.ends_with(b"\r\n--") {
                continue;
            }
            
            // exclude "\r\n\r\n" and "\r\n--"
            let payload_start = parameters.end + 4;
            let payload_end = current.len() - 4;
            
            let payload = &current[payload_start..payload_end];
            
            // -------------------- response --------------------
            
            return Some(payload);
            
        }
        
        None
        
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    use std::io::Cursor;
    
    #[cfg(test)]
    mod new {
        
        // headers_only
        // headers_and_body
        // body_not_signaled
        // body_without_length
        // body_with_case_mixed_length
        // body_with_lower_length
        // body_with_higher_length
        // body_with_excess_data
        // limit_exceeded_by_headers
        // limit_exceeded_by_body
        
        use super::*;
        
        #[test]
        fn headers_only() {
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
        fn headers_and_body() {
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
        fn body_not_signaled() {
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
        fn body_without_length() {
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
        fn body_with_case_mixed_length() {
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
        fn body_with_lower_length() {
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
        fn body_with_higher_length() {
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
        fn limit_exceeded_by_headers() {
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
        
        // with_query_string
        // without_query_string        
        // with_extra_whitespace
        // with_invalid_whitespace
        // without_method
        // without_target
        // without_version
        
        use super::*;
        
        #[test]
        fn with_query_string() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?fkey=fvalue HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"GET".as_slice());
            assert_eq!(target, b"/test/resource".as_slice());
        }
        
        #[test]
        fn without_query_string() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"GET".as_slice());
            assert_eq!(target, b"/test/resource".as_slice());
        }
        
        #[test]
        fn with_extra_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET  /test/resource  HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"GET".as_slice());
            assert_eq!(target, b"/test/resource".as_slice());
        }
        
        #[test]
        fn with_invalid_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET\t/test/resource  HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"GET\t/test/resource".as_slice());
            assert_eq!(target, b"HTTP/1.1".as_slice());
        }
        
        #[test]
        fn without_method() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"/test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"/test/resource".as_slice());
            assert_eq!(target, b"HTTP/1.1".as_slice());
        }
        
        #[test]
        fn without_target() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"GET".as_slice());
            assert_eq!(target, b"HTTP/1.1".as_slice());
        }
        
        #[test]
        fn without_version() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            let (method, target) = output.unwrap();
            
            assert_eq!(method, b"GET".as_slice());
            assert_eq!(target, b"/test/resource".as_slice());
        }
        
    }
    
    #[cfg(test)]
    mod header {
        
        // exact_match
        // duplicate
        // case_mixed_content
        // case_mixed_key
        // without_content_whitespace
        // with_extra_content_whitespace
        // with_tab_as_content_whitespace
        // with_extra_key_whitespace
        // without_value
        // key_not_present
        
        use super::*;
        
        #[test]
        fn exact_match() {
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
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"1");
        }
        
        #[test]
        fn duplicate() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Host: non-existant\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"host";
            
            // operation
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn case_mixed_content() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"hOsT: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"host";
            
            // operation
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn case_mixed_key() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"hOSt";
            
            // operation
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn without_content_whitespace() {
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
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"*/*");
        }
        
        #[test]
        fn with_extra_content_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:  */*  \r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Accept";
            
            // operation
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"*/*");
        }
        
        #[test]
        fn with_tab_as_content_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:\t*/*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Accept";
            
            // operation
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"*/*");
        }
        
        #[test]
        fn with_extra_key_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept : */*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let key = b"Accept";
            
            // operation
            
            let output = request.header(key);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn without_value() {
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
            
            let output = request.header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"");
        }
        
        #[test]
        fn key_not_present() {
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
            
            let output = request.header(key);
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
    #[cfg(test)]
    mod form_data {
        
        // single
        // multiple
        // malformed_payload
        // no_name
        // additional_filename_pair
        // filename_pair_only
        // case_mixed_pair
        // no_whitespace_between_pairs
        // nonexistent_param
        // empty_name
        // empty_payload
        // empty_content
        // additional_whitespace_in_boundary
        // additional_parameter_before_boundary
        // additional_parameter_after_boundary
        // quoted_boundary
        // case_mixed_boundary
        // no_boundary
        // wrong_boundary
        
        use super::*;
        
        #[test]
        fn single() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn multiple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 200\r\n");
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
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn malformed_payload() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 198\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_name() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 103\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data;\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_filename_pair() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 137\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; filename=\"test.txt\"; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"85\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"85".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn filename_pair_only() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 121\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; filename=\"input\";\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"85\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixed_pair() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 114\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"CONTENT-DISPOSITION: form-data; naMe=\"inPUt\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_whitespace_between_pairs() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 114\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition:form-data;name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn nonexistent_param() {
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
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"third";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_name() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 196\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"second";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_payload() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 199\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"10\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"second";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some([].as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_content() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 0\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_whitespace_in_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary = 9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_parameter_before_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; charset=utf-8; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn additional_parameter_after_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999; charset=utf-8\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn quoted_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=\"9999999999999999999999999999\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixed_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; BOUNDARY=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn wrong_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 116\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999998\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--");
            
            let request = Request::new(Cursor::new(content.clone())).unwrap();
            let param = b"input";
            
            // operation
            
            let mut output = request.form_data(param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
}
