pub struct Headers {
    content: Vec<u8>,
}

impl Headers {
    
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content,
        }
    }
    
    pub fn endpoint(&self) -> Option<&[u8]> {
        // GET /test/resource?fkey=fvalue HTTP/1.1\r\n
        // Host: placeholder\r\n
        
        // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
        let mut components = self.content.splitn(2, |&curr| curr == b'\r');
        
        // to the left of '\r' => GET /test/resource?fkey=fvalue HTTP/1.1
        let first_line = components.next()?;
        
        // TODO: replace with slice::rsplit_once in the future (https://github.com/rust-lang/rust/issues/112811)
        let mut components = first_line.rsplitn(2, |&curr| curr == b' ');
        
        // to the left of ' ' => GET /test/resource?fkey=fvalue
        let endpoint = components.nth(1)?;
        
        // strip '?', if any => GET /test/resource
        endpoint.split(|&curr| curr == b'?').next()
    }
    
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        // GET /test/resource?fkey=fvalue HTTP/1.1\r\n
        // Host: placeholder\r\n
        
        let range = chikuwa::delimited_range(&self.content, key, b"\r\n")?;
        
        // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
        let mut components = self.content[range].splitn(2, |&curr| curr == b':');
        
        // to the right of ':' =>  placeholder
        components.nth(1).map(<[u8]>::trim_ascii_start)
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
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
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
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
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
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
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
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
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
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
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET".as_slice()));
        }
        
        #[test]
        fn method_only() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
    #[cfg(test)]
    mod get {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"host";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_some());
            
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
            
            let headers = Headers::new(content);
            let key = b"Sec-GPC";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_some());
            
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
            
            let headers = Headers::new(content);
            let key = b"host";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_some());
            
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
            
            let headers = Headers::new(content);
            let key = b"hOSt";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_some());
            
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
            
            let headers = Headers::new(content);
            let key = b"Accept";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_some());
            
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
            
            let headers = Headers::new(content);
            let key = b"Accept";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_some());
            
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
            
            let headers = Headers::new(content);
            let key = b"Content-Length";
            
            // operation
            
            let output = headers.get(key);
            
            // control
            
            assert!(output.is_none());
        }
        
    }
    
}
