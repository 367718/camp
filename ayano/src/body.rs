use crate::Headers;

pub struct Body {
    content: Vec<u8>,
}

pub struct FormParams<'r, 'h, 'k> {
    content: &'r [u8],
    content_type: ContentType<'h>,
    key: &'k [u8],
}

enum ContentType<'h> {
    FormUrlencoded,
    FormData(&'h [u8]),
    Other,
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
    
    pub fn form_params<'r, 'h, 'k>(&'r self, headers: &'h Headers, key: &'k [u8]) -> FormParams<'r, 'h, 'k> {
        let content = &self.content;
        
        let content_type = match headers.get(b"Content-Type") {
            
            // Content-Type: application/x-www-form-urlencoded;charset=UTF-8
            Some(content) if content.starts_with(b"application/x-www-form-urlencoded") => ContentType::FormUrlencoded,
            
            // Content-Type: multipart/form-data;boundary="9999999999999999999999999999"
            Some(content) if content.starts_with(b"multipart/form-data") => {
                let (_, boundary) = chikuwa::insensitive_split_once(content, b"boundary=");
                ContentType::FormData(boundary)
            },
            
            _ => ContentType::Other,
            
        };
        
        FormParams {
            content,
            content_type,            
            key,
        }
    }
    
}

impl Iterator for FormParams<'_, '_, '_> {
    
    type Item = Vec<u8>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.content_type {
            
            ContentType::FormUrlencoded => {
                
                // fkey=fvalue
                // fkey=fvalue&skey=svalue
                
                while ! self.content.is_empty() {
                    
                    let (working, rest) = chikuwa::insensitive_split_once(self.content, b"&");
                    let (left, right) = chikuwa::insensitive_split_once(working, b"=");
                    
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
                
            },
            
            ContentType::FormData(boundary) => {
                
                // -----------------------------9999999999999999999999999999
                // Content-Disposition: form-data; name="placeholder key #1"
                // 
                // placeholder value #1
                // -----------------------------9999999999999999999999999999
                // Content-Disposition: form-data; name="placeholder key #2"
                // 
                // placeholder value #2
                // -----------------------------9999999999999999999999999999--
                
                while let Some(param) = chikuwa::subslice_range(self.content, boundary, boundary) {
                    
                    let (key, value) = build_form_data_pair(&self.content[param.start..param.end])?;
                    self.content = &self.content[param.end..];
                    
                    if key.is_empty() || value.is_empty() {
                        continue;
                    }
                    
                    if key.eq_ignore_ascii_case(self.key) {
                        return Some(value.to_vec());
                    }
                    
                }
                
                None
                
            },
            
            ContentType::Other => None,
            
        }
        
    }
    
}

fn build_form_data_pair(param: &[u8]) -> Option<(&[u8], &[u8])> {
    
    // Content-Disposition: form-data; name="placeholder key"
    // 
    // placeholder value
    // --
    
    let data = chikuwa::subslice_range(param, b"Content-Disposition: form-data; name=\"", b"\"\r\n\r\n")?;
    
    let key = &param[data.start..data.end];
    let value = param[data.end..][5..].strip_suffix(b"\r\n--")?;
    
    Some((key, value))
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    mod form_params {
        
        use super::*;
        
        mod url_encoded {
            
            use super::*;
            
            #[test]
            fn simple() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"key=value");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"key";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert_eq!(output.next().as_deref(), Some(b"value".as_slice()));
                assert!(output.next().is_none());
            }
            
            #[test]
            fn complex() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"fkey=fvalue&skey=svalue&tkey=tvalue");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"skey";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert_eq!(output.next().as_deref(), Some(b"svalue".as_slice()));
                assert!(output.next().is_none());
            }
            
            #[test]
            fn duplicate() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"key=value1&key=value2");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"key";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert_eq!(output.next().as_deref(), Some(b"value1".as_slice()));
                assert_eq!(output.next().as_deref(), Some(b"value2".as_slice()));
                assert!(output.next().is_none());
            }
            
            #[test]
            fn case_mixing() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"fkey=fvalue&skey=svalue");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"SKEY";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert_eq!(output.next().as_deref(), Some(b"svalue".as_slice()));
                assert!(output.next().is_none());
            }
            
            #[test]
            fn nonexistent_key() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"fkey=fvalue&skey=svalue");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"tkey";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert!(output.next().is_none());
            }
            
            #[test]
            fn empty_key() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"fkey=fvalue&=svalue");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"skey";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert!(output.next().is_none());
            }
            
            #[test]
            fn empty_value() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let mut body_content = Vec::new();
                body_content.extend_from_slice(b"fkey=fvalue&skey=");
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"skey";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert!(output.next().is_none());
            }
            
            #[test]
            fn no_pairs() {
                // setup
                
                let mut headers_content = Vec::new();
                headers_content.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
                headers_content.extend_from_slice(b"Host: placeholder\r\n");
                headers_content.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
                headers_content.extend_from_slice(b"\r\n");
                
                let body_content = Vec::new();
                
                let headers = Headers::new(headers_content);
                let body = Body::new(body_content);
                let key = b"key";
                
                // operation
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert!(output.next().is_none());
            }
            
        }
        
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
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
                
                let mut output = body.form_params(&headers, key);
                
                // control
                
                assert!(output.next().is_none());
            }
            
        }
        
    }
    
}
