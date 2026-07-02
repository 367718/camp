use crate::Headers;

pub struct Body {
    content: Vec<u8>,
}

pub struct FormData<'r, 'h, 'k> {
    content: &'r [u8],
    boundary: &'h [u8],
    key: &'k [u8],
}

impl Body {
    
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content,
        }
    }
    
    pub fn len(&self) -> usize {
        self.content.len()
    }
    
    pub fn form_data<'r, 'h, 'k>(&'r self, headers: &'h Headers, key: &'k [u8]) -> FormData<'r, 'h, 'k> {
        // Content-Type: multipart/form-data; boundary=9999999999999999999999999999
        let boundary = headers.get(b"Content-Type")
            .filter(|value| value.starts_with(b"multipart/form-data"))
            .map_or(&[] as &[u8], |value| chikuwa::split_once(value, b"boundary=").1);
        
        FormData {
            content: &self.content,
            boundary,
            key,
        }
    }
    
}

impl<'r> Iterator for FormData<'r, '_, '_> {
    
    type Item = &'r [u8];
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // -----------------------------9999999999999999999999999999
        // Content-Disposition: form-data; name="placeholder key #1"
        // 
        // placeholder value #1
        // -----------------------------9999999999999999999999999999
        // Content-Disposition: form-data; name="placeholder key #2"
        // 
        // placeholder value #2
        // -----------------------------9999999999999999999999999999--
        
        while let Some(param) = chikuwa::delimited_range(self.content, self.boundary, self.boundary) {
            
            let (key, value) = build_pair(&self.content[param])?;
            self.content = &self.content[param.end..];
            
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

fn build_pair(param: &[u8]) -> Option<(&[u8], &[u8])> {
    
    // Content-Disposition: form-data; name="placeholder key"
    // 
    // placeholder value
    // --
    
    let data = chikuwa::delimited_range(param, b"Content-Disposition: form-data; name=\"", b"\"\r\n\r\n")?;
    
    let key = &param[data];
    let value = param[data.end + 5..].strip_suffix(b"\r\n--")?;
    
    Some((key, value))
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod form_data {
        
        use super::*;
        
        #[test]
        fn simple() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"input";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn complex() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"10\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"second";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"10".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn duplicate() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"10\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"input";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"10".as_slice()));
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn case_mixing() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"inPUT";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert_eq!(output.next().as_deref(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn nonexistent_key() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"10\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"third";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_key() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"10\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"first";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_value() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"10\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"first";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_pairs() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let body_content = Vec::new();
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"input";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_boundary() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"input";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        
        
        #[test]
        fn wrong_boundary() {
            // setup
            
            let mut headers_content = Vec::new();
            headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
            headers_content.extend_from_slice(b"Host: placeholder\r\n");
            headers_content.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999998\r\n");
            headers_content.extend_from_slice(b"\r\n");
            
            let mut body_content = Vec::new();
            body_content.extend_from_slice(b"------9999999999999999999999999999\r\n");
            body_content.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
            body_content.extend_from_slice(b"\r\n");
            body_content.extend_from_slice(b"90\r\n");
            body_content.extend_from_slice(b"------9999999999999999999999999999--\r\n");
            
            let headers = Headers::new(headers_content);
            let body = Body::new(body_content);
            let key = b"input";
            
            // operation
            
            let mut output = body.form_data(&headers, key);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
}
