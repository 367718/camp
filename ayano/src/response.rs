use std::{
    io::{ self, Error, ErrorKind, Write, Cursor },
    mem,
    os::raw::c_void,
    ptr,
    thread,
};

use crate::{ sys, Server, ContentType, CacheControl };

pub struct Response<'s> {
    request_id: u64, // HTTP_OPAQUE_ID
    uncommitted: Vec<u8>,
    failed: bool,
    server: &'s Server,
}

const UNCOMMITTED_BUFFER_CAPACITY: usize = 16 * 1024;

impl<'s> Response<'s> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(server: &'s Server, request_id: u64, status: u16, content: ContentType, cache: CacheControl) -> io::Result<Self> {
        // SAFETY: all-zero bit patterns are valid for structs that consist entirely of integer primitives and raw pointers
        let mut headers: sys::HttpResponseHeaders = unsafe { mem::zeroed() };
        
        Self::add_header(&mut headers, sys::HTTP_HEADER_TRANSFER_ENCODING, b"chunked");
        Self::add_header(&mut headers, sys::HTTP_HEADER_CONTENT_TYPE, content.into_header());
        Self::add_header(&mut headers, sys::HTTP_HEADER_CACHE_CONTROL, cache.into_header());
        
        // SAFETY: all-zero bit patterns are valid for structs that consist entirely of integer primitives and raw pointers
        let mut response: sys::HttpResponseV2 = unsafe { mem::zeroed() };
        
        response.version = sys::HTTP_VERSION_1_1;
        response.status_code = status;
        response.headers = headers;
        
        // `HTTP_SEND_RESPONSE_FLAG_MORE_DATA`: additional data will be sent in subsequent calls for this response
        // `HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA`: enables buffering, which maximizes performance and throughput for small-to-medium responses
        let flags = sys::HTTP_SEND_RESPONSE_FLAG_MORE_DATA | sys::HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA;
        let mut bytes_sent: u32 = 0;
        
        // SAFETY:
        //  1. `server.queue_handle()` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&'s Server`
        //  2. `request_id` is a valid id obtained from `HttpReceiveHttpRequest` corresponding to a still-alive request
        //  3. `flags` is a valid option defined by the http.sys API
        //  4. `&raw const response` points to a properly aligned `HttpResponseV2` whose referenced header buffers are valid and readable
        //  5. `p_cache_policy` as `null` is valid and selects standard http.sys defaults
        //  6. `&raw mut bytes_sent` points to a live, properly aligned `u32` valid for writes
        //  7. `p_reserved1` as `null` is required by the http.sys API
        //  8. `reserved2` as `0` is required by the http.sys API
        //  9. `p_overlapped` as `null` ensures synchronous execution, stack references (`response`, `bytes_sent`) cannot dangle
        // 10. `p_log_data` as `null` is valid and selects standard http.sys defaults
        let result = unsafe {
            
            sys::HttpSendHttpResponse(
                server.queue_handle(),
                request_id,
                flags,
                &raw const response,
                ptr::null(), // no kernel caching
                &raw mut bytes_sent,
                ptr::null(), // reserved
                0, // reserved
                ptr::null_mut(), // synchronous call
                ptr::null(), // no logging
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        let uncommitted = Vec::new();
        let failed = false;
        
        Ok(Response { request_id, uncommitted, failed, server })
    }
    
    
    // -------------------- mutators --------------------
    
    
    fn disconnect(&mut self) {
        // SAFETY:
        // 1. `self.server.queue_handle()` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&'s Server`
        // 2. `self.request_id` is a valid id obtained from `HttpReceiveHttpRequest` corresponding to a still-alive request
        // 3. `p_overlapped` as `null` ensures synchronous execution
        unsafe {
            
            sys::HttpCancelHttpRequest(
                self.server.queue_handle(),
                self.request_id,
                ptr::null_mut(), // synchronous call
            )
            
        };
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn add_header(headers: &mut sys::HttpResponseHeaders, id: usize, value: &[u8]) {
        headers.known_headers[id] = sys::HttpKnownHeader {
            raw_value_length: u16::try_from(value.len()).expect("Unexpected truncation"),
            p_raw_value: value.as_ptr().cast::<i8>(),
        };
    }
    
    fn generate_chunk(buffer: &[u8]) -> sys::HttpDataChunk {
        sys::HttpDataChunk {
            data_chunk_type: sys::HTTP_DATA_CHUNK_FROM_MEMORY,
            p_buffer: buffer.as_ptr().cast::<c_void>(),
            buffer_length: u32::try_from(buffer.len()).expect("Unexpected truncation"),
            _padding: [0; 3],
        }
    }
    
    fn send_chunk(server: &Server, request_id: u64, data: &[u8], is_final: bool) -> io::Result<()> {
        // for a buffer of 16384 (16 * 1024) in length, the maximum possible value would be 6 bytes: 4000\r\n
        let mut header = [0_u8; 6];
        let mut cursor = Cursor::new(&mut header[..]);
        write!(cursor, "{:X}\r\n", data.len()).unwrap();
        let header_len = usize::try_from(cursor.position()).expect("Unexpected truncation");
        
        let trailer: &[u8] = if is_final {
            // signal the client that no more chunks will be sent by sending a final zero-sized chunk
            b"\r\n0\r\n\r\n"
        } else {
            b"\r\n"
        };
        
        let chunks = [
            Self::generate_chunk(&header[..header_len]),
            Self::generate_chunk(data),
            Self::generate_chunk(trailer),
        ];
        
        let flags = if is_final {
            // signal the kernel that no more chunks will be sent by omitting the `HTTP_SEND_RESPONSE_FLAG_MORE_DATA` flag
            // enables buffering, which maximizes performance and throughput for small-to-medium responses
            sys::HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA
        } else {
            // `HTTP_SEND_RESPONSE_FLAG_MORE_DATA`: additional data will be sent in subsequent calls for this response
            // `HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA`: enables buffering, which maximizes performance and throughput for small-to-medium responses
            sys::HTTP_SEND_RESPONSE_FLAG_MORE_DATA | sys::HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA
        };
        
        Self::send_entity_chunks(server, request_id, &chunks, flags)
    }
    
    fn send_entity_chunks(server: &Server, request_id: u64, chunks: &[sys::HttpDataChunk], flags: u32) -> io::Result<()> {
        let mut bytes_sent: u32 = 0;
        
        // SAFETY:
        //  1. `server.queue_handle()` is a valid request queue handle whose lifetime is guaranteed by the borrowed `&'s Server`
        //  2. `request_id` is a valid id obtained from `HttpReceiveHttpRequest` corresponding to a still-alive request
        //  3. `flags` is a valid option defined by the http.sys API
        //  4. `entity_chunk_count` as `chunks.len()` matches the number of `HttpDataChunk` structs to be sent
        //  5. `chunks.as_ptr()` points to a live, properly aligned array of `HttpDataChunk` structs to be sent
        //  6. `&raw mut bytes_sent` points to a live, properly aligned `u32` valid for writes
        //  7. `p_reserved1` as `null` is required by the http.sys API
        //  8. `reserved2` as `0` is required by the http.sys API
        //  9. `p_overlapped` as `null` ensures synchronous execution, stack references (`chunks`, `bytes_sent`) cannot dangle
        // 10. `p_log_data` as `null` is valid and selects standard http.sys defaults
        let result = unsafe {
            
            sys::HttpSendResponseEntityBody(
                server.queue_handle(),
                request_id,
                flags,
                u16::try_from(chunks.len()).expect("Unexpected truncation"),
                chunks.as_ptr(),
                &raw mut bytes_sent,
                ptr::null(), // reserved
                0, // reserved
                ptr::null_mut(), // synchronous call
                ptr::null(), // no logging
            )
            
        };
        
        if result != sys::NO_ERROR {
            return Err(Error::from_raw_os_error(result.cast_signed()));
        }
        
        Ok(())
    }
    
}

impl Write for Response<'_> {
    
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.failed {
            return Err(Error::new(ErrorKind::BrokenPipe, "Response is in an error state"));
        }
        
        if buf.is_empty() {
            return Ok(0);
        }
        
        if self.uncommitted.capacity() == 0 {
            self.uncommitted.reserve_exact(UNCOMMITTED_BUFFER_CAPACITY);
        }
        
        // copy as much as possible into uncommitted buffer
        let available = UNCOMMITTED_BUFFER_CAPACITY - self.uncommitted.len();
        let bytes = buf.len().min(available);
        self.uncommitted.extend_from_slice(&buf[..bytes]);
        
        // uncommitted buffer saturated, trigger flush
        if self.uncommitted.len() >= UNCOMMITTED_BUFFER_CAPACITY {
            self.flush()?;
        }
        
        Ok(bytes)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(Error::new(ErrorKind::BrokenPipe, "Response is in an error state"));
        }
        
        if self.uncommitted.is_empty() {
            return Ok(());
        }
        
        // clear uncommited buffer and keep transmission open
        if let Err(error) = Self::send_chunk(self.server, self.request_id, &self.uncommitted, false) {
            self.failed = true;
            return Err(error);
        }
        
        self.uncommitted.clear();
        
        Ok(())
    }
    
}

impl Drop for Response<'_> {
    
    fn drop(&mut self) {
        if thread::panicking() || self.failed {
            self.disconnect();
            return;
        }
        
        let result = if self.uncommitted.is_empty() {
            
            // signal the client that no more chunks will be sent by sending a final zero-sized chunk
            let chunk = Self::generate_chunk(b"0\r\n\r\n");
            
            // signal the kernel that no more chunks will be sent by omitting the `HTTP_SEND_RESPONSE_FLAG_MORE_DATA` flag
            // enables buffering, which maximizes performance and throughput for small-to-medium responses
            let flags = sys::HTTP_SEND_RESPONSE_FLAG_BUFFER_DATA;
            
            Self::send_entity_chunks(self.server, self.request_id, &[chunk], flags)
            
        } else {
            
            // clear uncommited buffer and end transmission
            Self::send_chunk(self.server, self.request_id, &self.uncommitted, true)
            
        };
        
        if result.is_err() {
            self.disconnect();
        }
    }
    
}
