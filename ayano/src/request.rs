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
    headers: Box<[u64]>,
    body: Vec<u8>,
    answered: bool,
    server: &'s Server,
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
        
        Ok(Self { headers, body, answered, server })
    }
    
    
    // -------------------- accessors --------------------
    
    
    pub fn verb(&self) -> Option<&'static str> {
        let headers_data = Self::headers_data(&self.headers);
        
        match headers_data.verb {
            sys::HTTP_VERB_GET => Some("GET"),
            sys::HTTP_VERB_POST => Some("POST"),
            _ => None,
        }
    }
    
    pub fn path(&self) -> Option<String> {
        let headers_data = Self::headers_data(&self.headers);
        let cooked_url = &headers_data.cooked_url;
        
        if cooked_url.p_abs_path.is_null() || cooked_url.abs_path_length == 0 {
            return None;
        }
        
        // SAFETY:
        // 1. `p_abs_path` was verified to be non-null and `abs_path_length` is greater than 0
        // 2. `p_abs_path` points to a valid, contiguous sequence of `abs_path_length` bytes located within `self.headers`
        // 3. `HttpCookedUrl` length fields are byte counts that, divided by `mem::size_of::<u16>()`, matches the element count expected by `slice::from_raw_parts`
        // 4. `path_utf16_slice` does not escape this function and is converted into an owned `String`
        let path_utf16_slice = unsafe {
            
            slice::from_raw_parts(
                cooked_url.p_abs_path,
                (cooked_url.abs_path_length as usize) / mem::size_of::<u16>(),
            )
            
        };
        
        String::from_utf16(path_utf16_slice).ok()
    }
    
    pub fn form_field<'r, 'p>(&'r self, param: &'p [u8]) -> FormField<'r, 'p> {
        let headers_data = Self::headers_data(&self.headers);
        let header_value = &headers_data.headers.known_headers[sys::HTTP_HEADER_CONTENT_TYPE];
        
        let content_type_raw_slice = if ! header_value.p_raw_value.is_null() && header_value.raw_value_length > 0 {
            
            // SAFETY:
            // 1. `p_raw_value` was verified to be non-null and `raw_value_length` is greater than 0
            // 2. `p_raw_value` points to a valid, contiguous sequence of `raw_value_length` bytes located within `self.headers`
            // 3. `HttpKnownHeader` length fields are byte counts, which matches the `u8` element count expected by `slice::from_raw_parts`
            // 4. `content_type_raw_slice` lifetime is bound to `'r`, during which `self.headers` remains alive and immutable
            Some(unsafe {
                
                slice::from_raw_parts(
                    header_value.p_raw_value.cast::<u8>(),
                    header_value.raw_value_length as usize,
                )
                
            })
            
        } else {
            
            None
            
        };
        
        FormField::new(&self.body, content_type_raw_slice, param)
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
        // boxed slice is used to ensure a stable address, since `HttpRequestV2` stores self-referential pointers
        // `u64` is used to ensure 8-byte alignment
        let mut buffer = vec![0_u64; REQUEST_HEADERS_SIZE_LIMIT].into_boxed_slice();
        let mut amount_read: c_ulong = 0;
        
        // SAFETY:
        // 1. `server.queue_handle()` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&Server`
        // 2. `request_id` as `0` is valid and selects standard http.sys defaults
        // 3. `flags` as `0` is valid and selects standard http.sys defaults
        // 4. `buffer.as_mut_ptr().cast::<c_void>()` points to a live, non-empty, properly aligned memory buffer valid for writes
        // 5. `size_of_val(&*buffer)` matches the size in bytes of `buffer`, preventing out-of-bounds writes
        // 6. `&raw mut amount_read` points to a live, properly aligned `c_ulong` valid for writes
        // 7. `p_overlapped` as `null` is valid and selects standard http.sys defaults
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
            
            // just enough data to get `request_id`
            if amount_read >= 24 {
                let request_id = Self::headers_data(&buffer).request_id;
                Self::abort(server, request_id, StatusCode::BadRequest);
            }
            
            return Err(Error::new(ErrorKind::InvalidData, "Malformed request"));
            
        }
        
        // -------------------- incompatible HTTP version --------------------
        
        let data = Self::headers_data(&buffer);
        
        // HTTP version requested is not supported
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
            
            // `chunk` will be filled with one or more entity bodies
            let flags = sys::HTTP_RECEIVE_REQUEST_ENTITY_BODY_FLAG_FILL_BUFFER;
            
            let mut bytes_read: c_ulong = 0;
            
            loop {
                
                // SAFETY:
                // 1. `server.queue_handle()` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&Server`
                // 2. `headers_data.request_id` is a valid request_id corresponding to a still-alive request
                // 3. `flags` is a valid option defined by the http.sys API
                // 4. `chunk.as_mut_ptr().cast::<c_void>()` points to a live, non-empty byte slice valid for writes
                // 5. `chunk_len` matches the size in bytes of `chunk`, preventing out-of-bounds writes
                // 6. `&raw mut bytes_read` points to a live, properly aligned `c_ulong` valid for writes
                // 7. `p_overlapped` as `null` is valid and selects standard http.sys defaults
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
        // SAFETY:
        // 1. `headers.as_ptr()` is non-null and aligned to 8 bytes (`u64`), which matches the 8-byte alignment required by `HttpRequestV2`
        // 2. the memory slice is guaranteed to be at least `size_of::<sys::HttpRequestV2>()` bytes in length
        // 3. all bytes are initialized and valid for integer primitives and raw pointers
        // 4. `&HttpRequestV2` lifetime is tied to `headers`
        unsafe { &*headers.as_ptr().cast::<sys::HttpRequestV2>() }
    }
    
    fn abort(server: &Server, request_id: c_ulonglong, status: StatusCode) {
        // SAFETY: all-zero bit patterns are valid for structs that consist entirely of integer primitives and raw pointers
        let mut response: sys::HttpResponseV2 = unsafe { mem::zeroed() };
        
        response.version = sys::HTTP_VERSION_1_1;
        response.status_code = status.into_int();
        
        // the network connection should be closed after sending this response
        let flags = sys::HTTP_SEND_RESPONSE_FLAG_DISCONNECT;
        let mut bytes_sent: c_ulong = 0;
        
        // SAFETY:
        //  1. `server.queue_handle()` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&Server`
        //  2. `request_id` is a valid request_id corresponding to a still-alive request
        //  3. `flags` is a valid option defined by the http.sys API
        //  4. `&raw mut response` points to a properly aligned, zero-initialized `HttpResponseV2` struct
        //  5. `p_cache_policy` as `null` is valid and selects standard http.sys defaults
        //  6. `&raw mut bytes_sent` points to a live, properly aligned `c_ulong` valid for writes
        //  7. `p_reserved1` as `null` is required by the http.sys API
        //  8. `reserved2` as `0` is required by the http.sys API
        //  9. `p_overlapped` as `null` is valid and selects standard http.sys defaults
        // 10. `p_log_data` as `null` is valid and selects standard http.sys defaults
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
            )
            
        };
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
