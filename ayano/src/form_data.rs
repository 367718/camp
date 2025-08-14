pub struct FormData<'r> {
    boundary: &'r [u8],
    content: &'r [u8],
}

struct FormDataIterator<'r> {
    boundary: &'r [u8],
    content: &'r [u8],
}

impl<'r> FormData<'r> {
    
    pub(crate) fn new(headers: &'r [u8], content: &'r [u8]) -> Option<Self> {
        let value = chikuwa::subslice_range(headers, b"Content-Type:", b"\r\n")?;
        let boundary = headers[value].trim_ascii_start()
            .strip_prefix(b"multipart/form-data; boundary=")?;
        
        Some(FormData {
            boundary,
            content,
        })
    }
    
    pub fn get<'g>(&'g self, query: &[u8]) -> impl Iterator<Item = &'g [u8]> {
        let iter = FormDataIterator {
            boundary: self.boundary,
            content: self.content,
        };
        
        iter.filter(move |(key, _)| *key == query)
            .map(|(_, value)| value)
    }
    
}

impl<'r> Iterator for FormDataIterator<'r> {
    
    type Item = (&'r [u8], &'r [u8]);
    
    fn next(&mut self) -> Option<Self::Item> {
        
        // example
        
        // -----------------------------9999999999999999999999999999
        // Content-Disposition: form-data; name="placeholder key #1"
        // 
        // placeholder value #1
        // -----------------------------9999999999999999999999999999
        // Content-Disposition: form-data; name="placeholder key #2"
        // 
        // placeholder value #2
        // -----------------------------9999999999999999999999999999--
        
        while let Some(param) = chikuwa::subslice_range(self.content, self.boundary, self.boundary) {
            
            let item = build_pair(&self.content[param.start..param.end]);
            self.content = &self.content[param.end..];
            
            if item.is_some() {
                return item;
            }
            
        }
        
        None
        
    }
    
}

fn build_pair(param: &[u8]) -> Option<(&[u8], &[u8])> {
    
    // example
    
    // Content-Disposition: form-data; name="placeholder"
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
    
    #[test]
    fn simple() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        let mut pairs = output.get(b"input");
        
        assert_eq!(pairs.next(), Some(b"90".as_slice()));
        assert!(pairs.next().is_none());
    }
    
    #[test]
    fn mutiple() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"10\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        let mut pairs = output.get(b"input");
        
        assert_eq!(pairs.next(), Some(b"10".as_slice()));
        assert_eq!(pairs.next(), Some(b"90".as_slice()));
        assert!(pairs.next().is_none());
    }
    
    #[test]
    fn two_names() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"second\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"10\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"first\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        
        let mut first = output.get(b"first");
        assert_eq!(first.next(), Some(b"90".as_slice()));
        assert!(first.next().is_none());
        
        let mut second = output.get(b"second");
        assert_eq!(second.next(), Some(b"10".as_slice()));
        assert!(second.next().is_none());
    }
    
    #[test]
    fn no_content() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999999\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let body = Vec::new();
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        
        assert!(output.get(b"input").next().is_none());
        
    }
    
    #[test]
    fn no_boundary() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_none());
    }
    
    #[test]
    fn wrong_boundary() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type: multipart/form-data; boundary=----9999999999999999999999999998\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        
        assert!(output.get(b"input").next().is_none());
    }
    
    #[test]
    fn no_space_boundary() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type:multipart/form-data; boundary=----9999999999999999999999999999\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        let mut pairs = output.get(b"input");
        
        assert_eq!(pairs.next(), Some(b"90".as_slice()));
        assert!(pairs.next().is_none());
    }
    
    #[test]
    fn multiple_space_boundary() {
        // setup
        
        let mut headers = Vec::new();
        headers.extend_from_slice(b"POST /test/endpoint HTTP/1.1\r\n");
        headers.extend_from_slice(b"Host: placeholder\r\n");
        headers.extend_from_slice(b"Content-Type:   multipart/form-data; boundary=----9999999999999999999999999999\r\n");
        headers.extend_from_slice(b"\r\n");
        
        let mut body = Vec::new();
        body.extend_from_slice(b"------9999999999999999999999999999\r\n");
        body.extend_from_slice(b"Content-Disposition: form-data; name=\"input\"\r\n");
        body.extend_from_slice(b"\r\n");
        body.extend_from_slice(b"90\r\n");
        body.extend_from_slice(b"------9999999999999999999999999999--\r\n");
        
        // operation
        
        let output = FormData::new(&headers, &body);
        
        // control
        
        assert!(output.is_some());
        
        let output = output.unwrap();
        let mut pairs = output.get(b"input");
        
        assert_eq!(pairs.next(), Some(b"90".as_slice()));
        assert!(pairs.next().is_none());
    }
    
}
