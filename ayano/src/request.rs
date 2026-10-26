use std::{
    io::{ self, Error, ErrorKind },
    mem,
    os::raw::*,
    ptr,
    slice,
};

use crate::{
    sys,
    Server, FormField, Response,
    StatusCode, ContentType, CacheControl,
};

pub struct Request<'s> {
    server: &'s Server,
    headers: Box<[u64]>,
    body: Vec<u8>,
    answered: bool,
}

const REQUEST_HEADERS_SIZE_LIMIT: usize = 16 * 128;
const REQUEST_BODY_SIZE_LIMIT: usize = 512 * 1024;

const REQUEST_BODY_CHUNK_CAPACITY: usize = 8 * 1024;

impl<'s> Request<'s> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(server: &'s Server) -> io::Result<Self> {
        let headers = Self::read_headers(server)?;
        let body = Self::read_body(server, &headers)?;
        let answered = false;
        
        Ok(Self { server, headers, body, answered })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn verb(&self) -> Option<&'static str> {
        let headers_data = Self::headers_data(&self.headers);
        
        let verb = match headers_data.verb {
            sys::HTTP_VERB_GET => "GET",
            sys::HTTP_VERB_POST => "POST",
            _ => return None,
        };
        
        Some(verb)
    }
    
    
    pub fn path(&self) -> Option<String> {
        let headers_data = Self::headers_data(&self.headers);
        
        let cooked = &headers_data.cooked_url;
        
        if cooked.p_abs_path.is_null() || cooked.abs_path_length == 0 {
            return None;
        }
        
        let path = unsafe {
            
            let bytes = slice::from_raw_parts(
                cooked.p_abs_path,
                (cooked.abs_path_length as usize) / mem::size_of::<u16>(),
            );
            
            String::from_utf16(bytes).ok()?
            
        };
        
        Some(path)
    }
    
    
    pub fn form_field<'r, 'p>(&'r self, param: &'p [u8]) -> FormField<'r, 'p> {
        let headers_data = Self::headers_data(&self.headers);
        let content_type_raw = &headers_data.headers.known_headers[sys::HTTP_HEADER_CONTENT_TYPE];
        
        let content_type = unsafe {
            
            if ! content_type_raw.p_raw_value.is_null() && content_type_raw.raw_value_length > 0 {
                
                let bytes = slice::from_raw_parts(
                    content_type_raw.p_raw_value.cast::<u8>(),
                    content_type_raw.raw_value_length as usize,
                );
                
                Some(bytes)
                
            } else {
                
                None
                
            }
            
        };
        
        FormField::new(&self.body, content_type, param)
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn start_response(&mut self, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Response<'s>> {
        if self.answered {
            return Err(Error::other("Response has already been sent"));
        }
        
        let request_id = Self::headers_data(&self.headers).request_id;
        let response = Response::new(self.server, request_id, status, content, cache)?;
        
        self.answered = true;
        
        Ok(response)
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn read_headers(server: &Server) -> io::Result<Box<[u64]>> {
        // ensure 8-byte alignment by using u64
        let mut buffer = vec![0_u64; REQUEST_HEADERS_SIZE_LIMIT].into_boxed_slice();
        let mut amount_read: c_ulong = 0;
        
        let result = unsafe {
            
            sys::HttpReceiveHttpRequest(
                server.queue_handle(),
                0,
                0,
                buffer.as_mut_ptr().cast::<c_void>(),
                c_ulong::try_from(mem::size_of_val(&*buffer)).expect("Unexpected truncation"),
                &raw mut amount_read,
                ptr::null_mut(),
            )
            
        };
        
        // -------------------- limit exceeded --------------------
        
        // capacity of the supplied buffer was insufficient
        if result == sys::ERROR_MORE_DATA {
            // data struct is partially populated
            let request_id = Self::headers_data(&buffer).request_id;
            Self::abort(server, request_id, StatusCode::RequestHeaderFieldsTooLarge);
            return Err(Error::new(ErrorKind::InvalidData, "Request headers exceeded maximum allowable size"));
        }
        
        // -------------------- error --------------------
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        // -------------------- incomplete request --------------------
        
        // data struct has not been populated properly
        if (amount_read as usize) < mem::size_of::<sys::HttpRequestV2>() {
            
            // just enough data to get request_id
            if amount_read >= 24 {
                let request_id = Self::headers_data(&buffer).request_id;
                Self::abort(server, request_id, StatusCode::BadRequest);
            }
            
            return Err(Error::new(ErrorKind::InvalidData, "Malformed request"));
            
        }
        
        // -------------------- incompatible http version --------------------
        
        let data = Self::headers_data(&buffer);
        
        // http version requested is not supported
        if data.version != sys::HTTP_VERSION_1_1 {
            Self::abort(server, data.request_id, StatusCode::VersionNotSupported);
            return Err(Error::other("Unsupported HTTP version"));
        }
        
        Ok(buffer)
    }
    
    
    fn read_body(server: &Server, headers: &[u64]) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        
        let headers_data = Self::headers_data(headers);
        
        if (headers_data.flags & sys::HTTP_REQUEST_FLAG_MORE_ENTITY_BODY_EXISTS) != 0 {
            
            // ---------------- content-length check ----------------
            
            let Ok(content_length) = Self::parse_content_length(headers_data) else {
                Self::abort(server, headers_data.request_id, StatusCode::BadRequest);
                return Err(Error::new(ErrorKind::InvalidData, "Malformed Content-Length header"));
            };
            
            if let Some(content_length) = content_length {
                if content_length > REQUEST_BODY_SIZE_LIMIT {
                    Self::abort(server, headers_data.request_id, StatusCode::ContentTooLarge);
                    return Err(Error::new(ErrorKind::InvalidData, "Request body exceeded maximum allowable size"));
                }
                
                buffer.reserve(content_length);
            }
            
            // ---------------- content streaming ----------------
            
            let mut chunk = [0_u8; REQUEST_BODY_CHUNK_CAPACITY];
            let chunk_len = c_ulong::try_from(chunk.len()).expect("Unexpected truncation");
            
            loop {
                
                let mut bytes_read: c_ulong = 0;
                
                let result = unsafe {
                    
                    sys::HttpReceiveRequestEntityBody(
                        server.queue_handle(),
                        headers_data.request_id,
                        sys::HTTP_RECEIVE_REQUEST_ENTITY_BODY_FLAG_FILL_BUFFER,
                        chunk.as_mut_ptr().cast::<c_void>(),
                        chunk_len,
                        &raw mut bytes_read,
                        ptr::null_mut(),
                    )
                    
                };
                
                match result {
                    
                    sys::ERROR_HANDLE_EOF => break,
                    
                    sys::NO_ERROR if bytes_read == 0 => break,
                    
                    sys::NO_ERROR => {
                        
                        // while a check for the value of the content-length header has been done already,
                        // the header itself may be absent and nothing stops the client from actually sending more data anyway
                        if buffer.len() + (bytes_read as usize) > REQUEST_BODY_SIZE_LIMIT {
                            Self::abort(server, headers_data.request_id, StatusCode::ContentTooLarge);
                            return Err(Error::new(ErrorKind::InvalidData, "Request body exceeded maximum allowable size"));
                        }
                        
                        buffer.extend_from_slice(&chunk[..bytes_read as usize]);
                        
                    },
                    
                    _ => {
                        
                        Self::abort(server, headers_data.request_id, StatusCode::Error);
                        return Err(Error::from_raw_os_error(result.cast_signed()));
                        
                    },
                    
                }
                
            }
            
        }
        
        Ok(buffer)
    }
    
    
    fn headers_data(headers: &[u64]) -> &sys::HttpRequestV2 {
        unsafe { &*headers.as_ptr().cast::<sys::HttpRequestV2>() }
    }
    
    
    fn parse_content_length(headers_data: &sys::HttpRequestV2) -> io::Result<Option<usize>> {
        let content_length_raw = &headers_data.headers.known_headers[sys::HTTP_HEADER_CONTENT_LENGTH];
        
        if content_length_raw.p_raw_value.is_null() || content_length_raw.raw_value_length == 0 {
            return Ok(None);
        }
        
        let content_length_bytes = unsafe {
            
            slice::from_raw_parts(
                content_length_raw.p_raw_value.cast::<u8>(),
                content_length_raw.raw_value_length as usize,
            )
            
        };
        
        let content_length_string = str::from_utf8(content_length_bytes)
            .map_err(|_| Error::from(ErrorKind::InvalidData))?;
        
        // TODO: replace with usize::from_ascii in the future (https://github.com/rust-lang/rust/issues/134821)
        let content_length_int = content_length_string
            .trim()
            .parse::<usize>()
            .map_err(|_| Error::from(ErrorKind::InvalidData))?;
        
        Ok(Some(content_length_int))
    }
    
    
    fn abort(server: &Server, request_id: c_ulonglong, status: StatusCode) {
        let mut response: sys::HttpResponseV2 = unsafe { mem::zeroed() };
        
        response.version = sys::HTTP_VERSION_1_1;
        response.status_code = status.into_int();
        
        let mut bytes_sent: c_ulong = 0;
        
        unsafe {
            
            sys::HttpSendHttpResponse(
                server.queue_handle(),
                request_id,
                sys::HTTP_SEND_RESPONSE_FLAG_DISCONNECT,
                &raw mut response,
                ptr::null_mut(),
                &raw mut bytes_sent,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
        }
    }
    
}

impl Drop for Request<'_> {
    
    fn drop(&mut self) {
        if self.answered {
            return;
        }
        
        // unaswered request; assume error and drop connection
        let request_id = Self::headers_data(&self.headers).request_id;
        Self::abort(self.server, request_id, StatusCode::Error);
    }
    
}
