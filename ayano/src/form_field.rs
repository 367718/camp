pub struct FormField<'r, 'p> {
    content: &'r [u8],
    delimiter: Vec<u8>,
    param: &'p [u8],
}

enum BoundaryKind<'r> {
    Intermediate(&'r [u8]),
    Final,
}

impl<'r, 'p> FormField<'r, 'p> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(content: &'r [u8], content_type: Option<&'r [u8]>, param: &'p [u8]) -> Self {
        // Content-Type: multipart/form-data; boundary=9999999999999999999999999999
        
        let boundary = content_type
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
                    .find(|(key, _)| key.eq_ignore_ascii_case(b"boundary"))
                    .map(|(_, value)| value)?;
                
                // optional surrounding quotes
                let unquoted = match value {
                    [b'"', inner @ .., b'"'] => inner,
                    _ => value,
                };
                
                Some(unquoted)
                
            })
            .unwrap_or(&[]);
        
        if boundary.is_empty() {
            return Self {
                content: &[],
                delimiter: Vec::new(),
                param,
            };
        }
        
        // --9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #1" filename="example.txt"\r\n
        // \r\n
        // placeholder value #1\r\n
        // --9999999999999999999999999999\r\n
        // Content-Disposition: form-data; name="placeholder key #2" filename="example.txt"\r\n
        // \r\n
        // placeholder value #2\r\n
        // --9999999999999999999999999999--
        
        let mut delimiter = Vec::with_capacity(boundary.len() + 4);
        delimiter.extend_from_slice(b"\r\n--");
        delimiter.extend_from_slice(boundary);
        
        let mut reader = Self {
            content,
            delimiter,
            param,
        };
        
        // first boundary could be one of two possibilities:
        // --9999999999999999999999999999 if no preamble
        // \r\n--9999999999999999999999999999, like the rest, if a preamble is present
        if reader.consume_first_boundary(boundary).is_none() {
            reader.content = &[];
        }
        
        reader
    }
    
    
    // -------------------- mutators --------------------
    
    
    fn consume_first_boundary(&mut self, boundary: &[u8]) -> Option<()> {
        let mut first_delimiter = Vec::with_capacity(boundary.len() + 2);
        first_delimiter.extend_from_slice(b"--");
        first_delimiter.extend_from_slice(boundary);
        
        let after_delimiter = if let Some(rest) = self.content.strip_prefix(&first_delimiter[..]) {
            // no preamble
            rest
        } else {
            // preamble present
            let index = Self::find_subslice(self.content, &self.delimiter)?;
            &self.content[index + self.delimiter.len()..]
        };
        
        // payload is empty, as body consists only of opening/closing boundary
        if after_delimiter.starts_with(b"--") {
            self.content = &[];
            return None;
        }
        
        // optional linear whitespace
        let mut after_padding = after_delimiter;
        while let [b' ' | b'\t', rest @ ..] = after_padding {
            after_padding = rest;
        }
        
        self.content = after_padding.strip_prefix(b"\r\n")?;
        
        Some(())
    }
    
    fn next_part(&mut self) -> Option<(&'r [u8], &'r [u8])> {
        if self.content.is_empty() {
            return None;
        }
        
        let header_end = Self::find_subslice(self.content, b"\r\n\r\n")?;
        let headers = &self.content[..header_end];
        let after_headers = &self.content[header_end + 4..];
        
        let mut search_offset = 0;
        
        // look for a boundary line
        while let Some(delimiter_index) = Self::find_subslice(&after_headers[search_offset..], &self.delimiter) {
            let delimiter_start = search_offset + delimiter_index;
            let after_delimiter = &after_headers[delimiter_start + self.delimiter.len()..];
            
            match Self::parse_boundary_line(after_delimiter) {
                
                Some(BoundaryKind::Final) => {
                    let payload = &after_headers[..delimiter_start];
                    self.content = &[];
                    return Some((headers, payload));
                },
                
                Some(BoundaryKind::Intermediate(rest)) => {
                    let payload = &after_headers[..delimiter_start];
                    self.content = rest;
                    return Some((headers, payload));
                },
                
                // false positive, advance search past candidate
                None => search_offset = delimiter_start + self.delimiter.len(),
                
            }
        }
        
        // no terminating boundary found
        self.content = &[];
        
        None
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() {
            return None;
        }
        
        haystack.windows(needle.len())
            .position(|window| window == needle)
    }
    
    fn parse_boundary_line(after_delimiter: &[u8]) -> Option<BoundaryKind<'_>> {
        // -------------------- final boundary --------------------
        
        if let Some(after_dashes) = after_delimiter.strip_prefix(b"--") {
            
            // optional linear whitespace
            let mut after_whitespace = after_dashes;
            while let [b' ' | b'\t', rest @ ..] = after_whitespace {
                after_whitespace = rest;
            }
            
            // either end-of-body or crlf
            if after_whitespace.is_empty() || after_whitespace.starts_with(b"\r\n") {
                return Some(BoundaryKind::Final);
            }
            
        }
        
        // -------------------- intermediate boundary --------------------
        
        // optional linear whitespace
        let mut after_whitespace = after_delimiter;
        while let [b' ' | b'\t', rest @ ..] = after_whitespace {
            after_whitespace = rest;
        }
        
        // crlf
        if let Some(rest) = after_whitespace.strip_prefix(b"\r\n") {
            return Some(BoundaryKind::Intermediate(rest));
        }
        
        // -------------------- false positive --------------------
        
        None
    }
    
}

impl<'r> Iterator for FormField<'r, '_> {
    
    type Item = &'r [u8];
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            
            let (headers, payload) = self.next_part()?;
            
            let relevant = headers
                .split(|&byte| byte == b'\n')
                .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
                .find(|line| line.get(..20).is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"content-disposition:")))
                .is_some_and(|disposition| {
                    
                    disposition.split(|&byte| byte == b';')
                        .filter_map(|parameter| {
                            
                            // TODO: replace with slice::split_once in the future (https://github.com/rust-lang/rust/issues/112811)
                            let mut pair = parameter.splitn(2, |&byte| byte == b'=')
                                .map(<[u8]>::trim_ascii);
                            
                            let key = pair.next().unwrap();
                            let value = pair.next()?;
                            
                            Some((key, value))
                            
                        })
                        .filter(|(key, _)| key.eq_ignore_ascii_case(b"name"))
                        .any(|(_, value)| {
                            
                            // optional surrounding quotes
                            let unquoted = match value {
                                [b'"', inner @ .., b'"'] => inner,
                                _ => value,
                            };
                            
                            // form field names are case-sensitive
                            unquoted == self.param
                            
                        })
                    
                });
            
            if relevant {
                return Some(payload);
            }
            
        }
    }
    
}

#[cfg(test)]
mod tests {
    
    // single
    // multiple
    // with_content_type
    // with_preamble_and_epilogue
    // with_linear_whitespace
    // header_without_whitespace
    // header_with_extra_whitespace
    // malformed_payload_first
    // malformed_payload_second
    // no_name
    // additional_filename_pair
    // filename_pair_only
    // mixed_case_pair_key
    // mixed_case_pair_value
    // no_whitespace_between_pairs
    // nonexistent_param
    // empty_name
    // empty_payload
    // empty_content
    // additional_whitespace_in_boundary
    // additional_parameter_before_boundary
    // additional_parameter_after_boundary
    // quoted_boundary
    // half_quoted_boundary
    // mixed_case_boundary
    // no_boundary
    // wrong_boundary
    // boundary_collision
    
    use super::*;
    
    #[test]
    fn single() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
            "10\r\n",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
    fn with_content_type() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "Content-Type: text/plain; charset=utf-8\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"other\"\r\n",
            "\r\n",
            "10\r\n",
            "--9999999999999999999999999999--",
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
    fn with_preamble_and_epilogue() {
        // setup
        
        let content: Vec<u8> = concat!(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\r\n",
            "Maecenas in velit pharetra, imperdiet nisi vitae, suscipit metus.\r\n",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--\r\n",
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
    fn with_linear_whitespace() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999   \r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "10\r\n",
            "--9999999999999999999999999999\t\r\n",
            "Content-Disposition: form-data; name=\"other\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
    fn header_without_whitespace() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data;boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn header_with_extra_whitespace() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"  multipart/form-data;  boundary=9999999999999999999999999999  ");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn malformed_payload_first() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "10",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
    fn malformed_payload_second() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"first\"\r\n",
            "\r\n--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"first";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn no_name() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data;\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn additional_filename_pair() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; filename=\"test.txt\"; name=\"input\"\r\n",
            "\r\n",
            "85\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"85".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn filename_pair_only() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; filename=\"input\";\r\n",
            "\r\n",
            "85\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn mixed_case_pair_key() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; naMe=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
    fn mixed_case_pair_value() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"inPut\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn no_whitespace_between_pairs() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition:form-data;name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
            "10\r\n",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"first\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"third";
        
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
            "10\r\n",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"second\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
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
    fn empty_payload() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"first\"\r\n",
            "\r\n",
            "10\r\n",
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"second\"\r\n",
            "\r\n",
            "\r\n",
            "--9999999999999999999999999999--",
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
    fn additional_whitespace_in_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary = 9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn additional_parameter_before_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; charset=utf-8; boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn additional_parameter_after_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999; charset=utf-8");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn quoted_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=\"9999999999999999999999999999\"");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn half_quoted_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=\"9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn mixed_case_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; BOUNDARY=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"90".as_slice()));
        assert!(output.next().is_none());
    }
    
    #[test]
    fn no_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type = None;
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn wrong_boundary() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "90\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999998");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert!(output.next().is_none());
    }
    
    #[test]
    fn boundary_collision() {
        // setup
        
        let content: Vec<u8> = concat!(
            "--9999999999999999999999999999\r\n",
            "Content-Disposition: form-data; name=\"input\"\r\n",
            "\r\n",
            "ab--9999999999999999999999999999cd\r\n",
            "--9999999999999999999999999999--",
        ).as_bytes().to_vec();
        
        let content_type: Option<&[u8]> = Some(b"multipart/form-data; boundary=9999999999999999999999999999");
        let param = b"input";
        
        // operation
        
        let mut output = FormField::new(&content, content_type, param);
        
        // control
        
        assert_eq!(output.next(), Some(b"ab--9999999999999999999999999999cd".as_slice()));
        assert!(output.next().is_none());
    }
    
}
