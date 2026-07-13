use std::{
    io::{ self, Read, Error, ErrorKind },
    net::TcpStream,
};

use super::{
    REQUEST_SIZE_LIMIT, CONNECTION_BUFFER_SIZE,
    StatusCode, ContentType, CacheControl,
    Response,
};

pub struct Request {
    headers: Vec<u8>,
    body: Vec<u8>,
    stream: Option<TcpStream>,
}

pub struct FormData<'r, 'h, 'k> {
    content: &'r [u8],
    boundary: &'h [u8],
    key: &'k [u8],
}

impl Request {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(mut stream: TcpStream) -> io::Result<Self> {
        let (headers, body) = Self::get_headers_and_body(&mut stream)?;
        let stream = Some(stream);
        
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
    
    pub fn form_data<'r: 'h, 'h, 'k>(&'r self, key: &'k [u8]) -> FormData<'r, 'h, 'k> {
        // Content-Type: multipart/form-data; boundary=9999999999999999999999999999
        
        // multipart/form-data; boundary=9999999999999999999999999999
        let boundary = self.get_header(b"Content-Type")
            //  boundary=9999999999999999999999999999
            .and_then(|value| value.strip_prefix(b"multipart/form-data;"))
            // boundary=9999999999999999999999999999
            .map(<[u8]>::trim_ascii_start)
            // 9999999999999999999999999999
            .and_then(|value| value.strip_prefix(b"boundary="))
            .unwrap_or(&[]);
        
        FormData {
            content: &self.body,
            boundary,
            key,
        }
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response> {
        let stream = self.stream.take()
            .ok_or(Error::other("Response already sent"))?;
        
        Response::new(stream, status, content, cache)
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn get_headers_and_body(reader: &mut impl Read) -> io::Result<(Vec<u8>, Vec<u8>)> {
        let mut reader = reader.take(REQUEST_SIZE_LIMIT);
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
            
        };
        
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
        
        // -------------------- response --------------------
        
        Ok((headers, body))
    }
    
}

impl<'r> Iterator for FormData<'r, '_, '_> {
    
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
            
            let data = chikuwa::delimited_range(current, b"Content-Disposition: form-data; name=\"", b"\"\r\n\r\n")?;
            
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
    mod get_headers_and_body {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            // operation
            
            let output = Request::get_headers_and_body(&mut content.as_slice());
            
            // control
            
            let (headers, body) = output.unwrap();
            
            let control = headers
                .into_iter()
                .chain(body)
                .collect::<Vec<u8>>();
            
            assert_eq!(content, control);
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
            // control
            
            let (headers, body) = output.unwrap();
            
            let control = headers
                .into_iter()
                .chain(body)
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
            // control
            
            let (headers, body) = output.unwrap();
            
            let control = headers
                .into_iter()
                .chain(body)
                .collect::<Vec<u8>>();
            
            assert_eq!(content, control);
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
            // control
            
            let (_, body) = output.unwrap();
            
            assert_eq!(b"123".as_slice(), body);
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
            // control
            
            let (_, body) = output.unwrap();
            
            assert_eq!(b"1234".as_slice(), body);
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
            // control
            
            let (_, body) = output.unwrap();
            
            assert!(body.is_empty());
        }
        
        #[test]
        fn limit_exceeded() {
            use std::io::Write;
            
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
            
            let output = Request::get_headers_and_body(&mut &content[..]);
            
            // control
            
            let (headers, body) = output.unwrap();
            
            assert_eq!(REQUEST_SIZE_LIMIT as usize, headers.len() + body.len());
        }
        
    }
    
    #[cfg(test)]
    mod endpoint {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET /test/resource".as_slice()));
        }
        
        #[test]
        fn with_query_string() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?fkey=fvalue HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET /test/resource".as_slice()));
        }
        
        #[test]
        fn extra_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource  HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET /test/resource ".as_slice()));
        }
        
        #[test]
        fn without_method() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"/test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"/test/resource".as_slice()));
        }
        
        #[test]
        fn without_path() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            
            // operation
            
            let output = request.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET".as_slice()));
        }
        
        #[test]
        fn method_only() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            
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
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"host";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn complex() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"Sec-GPC";
            
            // operation
            
            let output = request.get_header(key);
            
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"host";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn case_mixing() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"hOSt";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"placeholder");
        }
        
        #[test]
        fn extra_whitespace() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:    */*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"Accept";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"*/*");
        }
        
        #[test]
        fn empty() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept:\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"Accept";
            
            // operation
            
            let output = request.get_header(key);
            
            // control
            
            let value = output.unwrap();
            
            assert_eq!(value, b"");
        }
        
        #[test]
        fn non_existant() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"User-Agent: Mozilla/9.0 (Windows NT 9.0; Win64; x64; rv:9.0) Gecko/9 Firefox/9.0\r\n");
            content.extend_from_slice(b"Accept: */*\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn complex() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"second";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"10".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn duplicate() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"10".as_slice()));
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixing() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"inPUT";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn nonexistent_key() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"third";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_key() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"first";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_value() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"first";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_pairs() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 0\r\n");
            content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_whitespace_in_boundary() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"Content-Length: 118\r\n");
            content.extend_from_slice(b"Content-Type:multipart/form-data;boundary=9999999999999999999999999999\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999\r\n");
            content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            content.extend_from_slice(b"\r\n");
            content.extend_from_slice(b"90\r\n");
            content.extend_from_slice(b"--9999999999999999999999999999--\r\n");
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_boundary() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn wrong_boundary() {
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
            
            let (headers, body) = Request::get_headers_and_body(&mut content.as_slice()).unwrap();
            let request = Request { headers, body, stream: None };
            let key = b"input";
            
            // operation
            
            let mut output = request.form_data(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
}
