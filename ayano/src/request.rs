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

const REQUEST_BODY_CHUNK_CAPACITY: usize = 16 * 1024;

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
        
        // "cooked.abs_path_length" is the length of the string in bytes (without the null terminator)
        // "slice::from_raw_parts" expects an element count
        let utf16_slice = unsafe {
            
            slice::from_raw_parts(
                cooked.p_abs_path,
                (cooked.abs_path_length as usize) / mem::size_of::<u16>(),
            )
            
        };
        
        String::from_utf16(utf16_slice).ok()
    }
    
    pub fn form_field<'r, 'p>(&'r self, param: &'p [u8]) -> FormField<'r, 'p> {
        let headers_data = Self::headers_data(&self.headers);
        let content_type_raw = &headers_data.headers.known_headers[sys::HTTP_HEADER_CONTENT_TYPE];
        
        let content_type = {
            
            if ! content_type_raw.p_raw_value.is_null() && content_type_raw.raw_value_length > 0 {
                
                Some(unsafe {
                    
                    slice::from_raw_parts(
                        content_type_raw.p_raw_value.cast::<u8>(),
                        content_type_raw.raw_value_length as usize,
                    )
                    
                })
                
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
        // box is used to ensure a stable address, since the "HttpRequestV2" struct stores self-referential pointers
        // u64 is used to ensure 8-byte alignment
        let mut buffer = vec![0_u64; REQUEST_HEADERS_SIZE_LIMIT].into_boxed_slice();
        let mut amount_read: c_ulong = 0;
        
        let result = unsafe {
            
            sys::HttpReceiveHttpRequest(
                server.queue_handle(),
                0, // retrieve next available request
                0, // retrieve only request headers
                buffer.as_mut_ptr().cast::<c_void>(),
                c_ulong::try_from(size_of_val(&*buffer)).expect("Unexpected truncation"),
                &raw mut amount_read,
                ptr::null_mut(), // synchronous call
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
            
            let mut chunk = [0_u8; REQUEST_BODY_CHUNK_CAPACITY];
            let chunk_len = c_ulong::try_from(chunk.len()).expect("Unexpected truncation");
            
            // chunk will be filled with one or more entity bodies
            let flags = sys::HTTP_RECEIVE_REQUEST_ENTITY_BODY_FLAG_FILL_BUFFER;
            
            let mut bytes_read: c_ulong = 0;
            
            loop {
                
                let result = unsafe {
                    
                    sys::HttpReceiveRequestEntityBody(
                        server.queue_handle(),
                        headers_data.request_id,
                        flags,
                        chunk.as_mut_ptr().cast::<c_void>(),
                        chunk_len,
                        &raw mut bytes_read,
                        ptr::null_mut(), // synchronous call
                    )
                    
                };
                
                match result {
                    
                    sys::ERROR_HANDLE_EOF => break,
                    
                    sys::NO_ERROR if bytes_read == 0 => break,
                    
                    sys::NO_ERROR => {
                        
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
    
    fn abort(server: &Server, request_id: c_ulonglong, status: StatusCode) {
        let mut response: sys::HttpResponseV2 = unsafe { mem::zeroed() };
        
        response.version = sys::HTTP_VERSION_1_1;
        response.status_code = status.into_int();
        
        // the network connection should be closed after sending this response
        let flags = sys::HTTP_SEND_RESPONSE_FLAG_DISCONNECT;
        let mut bytes_sent: c_ulong = 0;
        
        unsafe {
            
            sys::HttpSendHttpResponse(
                server.queue_handle(),
                request_id,
                flags,
                &raw mut response,
                ptr::null_mut(), // no kernel caching
                &raw mut bytes_sent,
                ptr::null_mut(), // reserved
                0, // reserved
                ptr::null_mut(), // synchronous call
                ptr::null_mut(), // no logging
            );
            
        }
    }
    
}

impl Drop for Request<'_> {
    
    fn drop(&mut self) {
        if self.answered {
            return;
        }
        
        // unaswered request; assume error and close connection
        let request_id = Self::headers_data(&self.headers).request_id;
        Self::abort(self.server, request_id, StatusCode::Error);
    }
    
}
