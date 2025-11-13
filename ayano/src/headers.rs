pub struct Headers {
    content: Vec<u8>,
}

pub struct QueryString<'r, 'k> {
    content: &'r [u8],
    key: &'k [u8],
}

impl Headers {
    
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content,
        }
    }
    
    pub fn endpoint(&self) -> Option<&[u8]> {
        // GET /test/resource?fkey=fvalue HTTP/1.1
        let (working, _) = chikuwa::split_slice_once(&self.content, b"\r");
        let mut parts = working.rsplitn(2, |&curr| curr == b' ');
        
        // HTTP/1.1
        parts.next()?;
        
        // GET /test/resource?fkey=fvalue
        let endpoint = parts.next()?;
        
        // strip query component, if any
        endpoint.split(|&curr| curr == b'?').next()
    }
    
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let range = chikuwa::subslice_range(&self.content, key, b"\r\n")?;
        
        let value = &self.content[range];
        let value = value.strip_prefix(b":").unwrap_or(value);
        let value = value.trim_ascii_start();
        
        Some(value)
    }
    
    pub fn query_string<'r, 'k>(&'r self, key: &'k [u8]) -> QueryString<'r, 'k> {
        // GET /test/resource?fkey=fvalue HTTP/1.1\r\n
        // GET /test/resource?fkey=fvalue&skey=svalue HTTP/1.1\r\n
        
        let mut content: &[u8] = &[];
        
        // first line
        let (working, _) = chikuwa::split_slice_once(&self.content, b"\r");
        
        if let Some(range) = chikuwa::subslice_range(working, b"?", b" ") {
            content = &working[range];
        }
        
        QueryString {
            content,
            key,
        }
    }
    
}

impl Iterator for QueryString<'_, '_> {
    
    type Item = Vec<u8>;
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // fkey=fvalue
        // fkey=fvalue&skey=svalue
        
        while ! self.content.is_empty() {
            
            let (working, rest) = chikuwa::split_slice_once(self.content, b"&");
            let (left, right) = chikuwa::split_slice_once(working, b"=");
            
            self.content = rest;
            
            if left.is_empty() || right.is_empty() {
                continue;
            }
            
            let mut key = Vec::new();
            chikuwa::percent_decode(left, &mut key).unwrap();
            
            if key.eq_ignore_ascii_case(self.key) {
                let mut value = Vec::new();
                chikuwa::percent_decode(right, &mut value).unwrap();
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
            content.extend_from_slice(b"GET  /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            
            // operation
            
            let output = headers.endpoint();
            
            // control
            
            assert_eq!(output, Some(b"GET  /test/resource".as_slice()));
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
    
    #[cfg(test)]
    mod query_string {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?key=value HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"value".to_vec()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn complex() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?key1=value1&key2=value2&key3=value3 HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key2";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"value2".to_vec()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn duplicate() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?key1=value1&key1=value2 HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key1";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"value1".to_vec()));
            assert_eq!(output.next(), Some(b"value2".to_vec()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixing() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?key=value HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"KEy";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert_eq!(output.next(), Some(b"value".to_vec()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn nonexistent_key() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?key1=value1&key2=value2&key3=value3 HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key4";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_key() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?=value HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_value() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource?key= HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn double_question_mark() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource??key=value HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn lone_question_mark() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource? HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_pairs() {
            // setup
            
            let mut content = Vec::new();
            content.extend_from_slice(b"GET /test/resource HTTP/1.1\r\n");
            content.extend_from_slice(b"Host: placeholder\r\n");
            content.extend_from_slice(b"\r\n");
            
            let headers = Headers::new(content);
            let key = b"key";
            
            // operation
            
            let mut output = headers.query_string(key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
}
