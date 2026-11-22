pub struct FormField<'r, 'p> {
    content: &'r [u8],
    delimiter: Vec<u8>,
    param: &'p [u8],
}

impl<'r, 'p> FormField<'r, 'p> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(content: &'r [u8], content_type: Option<&'r [u8]>, param: &'p [u8]) -> Self {
        // the client is expected to follow the WHATWG HTML multipart/form-data serialization spec
        // https://html.spec.whatwg.org/#multipart-form-data
        
        let boundary = content_type
            .and_then(|data| data.strip_prefix(b"multipart/form-data; boundary="));
        
        let Some(boundary) = boundary else {
            // an absent boundary is not allowed to match any content, so the iterator will yield nothing
            return Self {
                content: &[],
                delimiter: Vec::new(),
                param
            };
        };
        
        // --9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #1"\r\n
        // \r\n
        // placeholder value #1
        // \r\n--9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #2"\r\n
        // \r\n
        // placeholder value #2
        // \r\n--9999999999999999999999999999--\r\n
        
        // delimiters are in the form "\r\n--9999999999999999999999999999\r\n"
        // except the first one that lacks the "\r\n" prefix and the last one that has a "--\r\n" suffix
        let mut delimiter = Vec::with_capacity(boundary.len() + 4);
        delimiter.extend_from_slice(b"\r\n--");
        delimiter.extend_from_slice(boundary);
        
        // prepare the content for iteration by discarding the first delimiter
        let content = content
            .strip_prefix(&delimiter[2..])
            .and_then(|rest| rest.strip_prefix(b"\r\n"))
            .unwrap_or(&[]);
        
        Self { content, delimiter, param }
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn find_boundary<'h>(haystack: &'h [u8], delimiter: &[u8]) -> Option<(usize, &'h [u8])> {
        let mut offset = 0;
        
        while let Some(position) = Self::find_subslice(&haystack[offset..], delimiter) {
            
            let start = offset + position;
            let after = &haystack[start + delimiter.len()..];
            
            match after.get(..2)? {
                
                // intermediate delimiter
                b"\r\n" => return Some((start, &after[2..])),
                
                // final delimiter
                b"--" => return Some((start, &[])),
                
                // collision, keep searching
                _ => offset = start + delimiter.len(),
                
            }
            
        }
        
        None
    }
    
    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() {
            return None;
        }
        
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
    
}

impl<'r> Iterator for FormField<'r, '_> {
    
    type Item = &'r [u8];
    
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((boundary_start, rest)) = Self::find_boundary(self.content, &self.delimiter) {
            
            let part = &self.content[..boundary_start];
            self.content = rest;
            
            // Content-Disposition: form-data; name="placeholder key #1"\r\n
            let payload = part
                .strip_prefix(b"Content-Disposition: form-data; name=\"")
                .and_then(|part| part.strip_prefix(self.param))
                .and_then(|part| part.strip_prefix(b"\"\r\n\r\n"));
            
            if payload.is_some() {
                return payload;
            }
            
        }
        
        self.content = &[];
        
        None
    }
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod general {
        
        // single
        // multiple
        // trailing_data
        // nonexistent_param
        // empty_content
        // multipart_missing
        
        use super::*;
        
        #[test]
        fn single() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn multiple() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn trailing_data() {
            // setup
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
                "In auctor mi vel metus porttitor, ut sagittis mi malesuada.\r\n",
                "Aliquam ornare odio nec lobortis tincidunt.\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn nonexistent_param() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"second\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"first\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"third";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_content() {
            // setup
            
            let content: Vec<u8> = Vec::new();
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn multipart_missing() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
    #[cfg(test)]
    mod header {
        
        // no_name
        // file_upload
        // pair_separator_in_name
        // filename_pair_only
        // mixed_case_pair_value
        // empty_name
        // disposition_with_leading_whitespace
        // disposition_with_trailing_whitespace
        // disposition_missing
        // absent
        // form_data_missing
        // no_crlf_before_next_header
        
        use super::*;
        
        #[test]
        fn no_name() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data;\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn file_upload() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"; filename=\"test.txt\"\r\n",
                "Content-Type: text/plain\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn pair_separator_in_name() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"in;put\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"in;put";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn filename_pair_only() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; filename=\"input\";\r\n",
                "\r\n",
                "85",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn mixed_case_pair_value() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"inPut\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty_name() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"second\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"second";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn disposition_with_leading_whitespace() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "  Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn disposition_with_trailing_whitespace() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition  : form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn disposition_missing() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Type: text/plain\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn absent() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn form_data_missing() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: attachment; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn no_crlf_before_next_header() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"90".as_slice()));
            assert!(output.next().is_none());
        }
        
    }
    
    #[cfg(test)]
    mod payload {
        
        // malformed_first
        // malformed_second
        // empty
        // truncated
        // value_absent
        
        use super::*;
        
        #[test]
        fn malformed_first() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "10",
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"10--9999999999999999999999999999\r\nContent-Disposition: form-data; name=\"input\"\r\n\r\n90".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn malformed_second() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn empty() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"first\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"second\"\r\n",
                "\r\n",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"second";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some([].as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn truncated() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "10",
                "\r\n--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"10".as_slice()));
            assert!(output.next().is_none());
        }
        
        #[test]
        fn value_absent() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"first\"\r\n",
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"second\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"first";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
    }
    
    #[cfg(test)]
    mod boundary {
        
        // absent
        // mismatch
        // collision
        
        use super::*;
        
        #[test]
        fn absent() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type = None;
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn mismatch() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "90",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999998");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert!(output.next().is_none());
        }
        
        #[test]
        fn collision() {
            // setup
            
            let content: Vec<u8> = concat!(
                "--9999999999999999999999999999\r\n",
                "Content-Disposition: form-data; name=\"input\"\r\n",
                "\r\n",
                "ab\r\n--9999999999999999999999999999cd",
                "\r\n--9999999999999999999999999999--\r\n",
            ).as_bytes().to_vec();
            
            let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
            let param = b"input";
            
            // operation
            
            let mut output = FormField::new(&content, content_type, param);
            
            // control
            
            assert_eq!(output.next(), Some(b"ab\r\n--9999999999999999999999999999cd".as_slice()));
            assert!(output.next().is_none());
        }
        
    }
    
}
