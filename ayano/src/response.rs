use std::{
    io::{ self, Error, ErrorKind, Write },
    mem,
    os::raw::*,
    ptr,
    thread,
};

use crate::{
    sys,
    Server,
    StatusCode, ContentType, CacheControl,
};

pub struct Response<'s> {
    server: &'s Server,
    request_id: c_ulonglong, // HTTP_OPAQUE_ID
    uncommitted: Vec<u8>,
    failed: bool,
}

const UNCOMMITTED_BUFFER_CAPACITY: usize = 8 * 1024;

impl<'s> Response<'s> {
    
    // -------------------- constructors --------------------
    
    
    pub(crate) fn new(server: &'s Server, request_id: c_ulonglong, status: StatusCode, content: ContentType, cache: CacheControl) -> io::Result<Self> {
        let mut headers: sys::HttpResponseHeaders = unsafe { mem::zeroed() };
        
        Self::add_header(&mut headers, sys::HTTP_HEADER_TRANSFER_ENCODING, b"chunked");
        Self::add_header(&mut headers, sys::HTTP_HEADER_CONTENT_TYPE, content.into_header());
        Self::add_header(&mut headers, sys::HTTP_HEADER_CACHE_CONTROL, cache.into_header());
        
        let mut response: sys::HttpResponseV2 = unsafe { mem::zeroed() };
        
        response.version = sys::HTTP_VERSION_1_1;
        response.status_code = status.into_int();
        response.headers = headers;
        
        let mut bytes_sent: c_ulong = 0;
        
        unsafe {
            
            let result = sys::HttpSendHttpResponse(
                server.queue_handle(),
                request_id,
                sys::HTTP_SEND_RESPONSE_FLAG_MORE_DATA,
                &raw mut response,
                ptr::null_mut(),
                &raw mut bytes_sent,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
        };
        
        let uncommitted = Vec::with_capacity(UNCOMMITTED_BUFFER_CAPACITY);
        let failed = false;
        
        Ok(Response { server, request_id, uncommitted, failed })
    }
    
    
    // -------------------- mutators --------------------
    
    
    fn disconnect(&mut self) {
        let mut bytes_sent: c_ulong = 0;
        
        unsafe {
            
            sys::HttpSendResponseEntityBody(
                self.server.queue_handle(),
                self.request_id,
                sys::HTTP_SEND_RESPONSE_FLAG_DISCONNECT,
                0,
                ptr::null_mut(),
                &raw mut bytes_sent,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
        }
    }
    
    
    // -------------------- helpers --------------------
    
    
    fn add_header(headers: &mut sys::HttpResponseHeaders, id: usize, value: &[u8]) {
        headers.known_headers[id] = sys::HttpKnownHeader {
            raw_value_length: c_ushort::try_from(value.len()).expect("Unexpected truncation"),
            p_raw_value: value.as_ptr().cast::<c_char>(),
        };
    }
    
    
    fn generate_chunk(buffer: &[u8]) -> sys::HttpDataChunk {
        let mut chunk: sys::HttpDataChunk = unsafe { mem::zeroed() };
        
        chunk.data_chunk_type = sys::HTTP_DATA_CHUNK_FROM_MEMORY;
        chunk.union_data.from_memory = sys::HttpDataChunkFromMemory {
            p_buffer: buffer.as_ptr() as *mut c_void,
            buffer_length: c_ulong::try_from(buffer.len()).expect("Unexpected truncation"),
        };
        
        chunk
    }
    
    
    fn send_chunk(queue_handle: *mut c_void, request_id: c_ulonglong, payload: &[u8]) -> io::Result<()> {
        // an empty payload would signal the end of the transmission
        if payload.is_empty() {
            return Ok(());
        }
        
        // for a buffer of 8192 in length, the maximum possible value would be 6 bytes: 2000\r\n
        // 16 bytes is used as a safety margin
        let mut header = [0_u8; 16];
        let mut cursor = std::io::Cursor::new(&mut header[..]);
        write!(cursor, "{:X}\r\n", payload.len()).unwrap();
        let header_len = usize::try_from(cursor.position()).expect("Unsupported platform");
        
        let mut chunks = [
            Self::generate_chunk(&header[..header_len]),
            Self::generate_chunk(payload),
            Self::generate_chunk(b"\r\n"),
        ];
        
        let mut bytes_sent: c_ulong = 0;
        
        unsafe {
            
            let result = sys::HttpSendResponseEntityBody(
                queue_handle,
                request_id,
                sys::HTTP_SEND_RESPONSE_FLAG_MORE_DATA,
                c_ushort::try_from(chunks.len()).expect("Unexpected truncation"),
                chunks.as_mut_ptr(),
                &raw mut bytes_sent,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            if result != sys::NO_ERROR {
                return Err(Error::from_raw_os_error(result.cast_signed()));
            }
            
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
        
        // uncommitted buffer is empty and incoming data fills or exceeds its capacity, skip copy
        if self.uncommitted.is_empty() && buf.len() >= UNCOMMITTED_BUFFER_CAPACITY {
            
            let bytes = buf.len().min(c_ulong::MAX as usize);
            
            if let Err(error) = Self::send_chunk(self.server.queue_handle(), self.request_id, &buf[..bytes]) {
                self.failed = true;
                return Err(error);
            }
            
            return Ok(bytes);
            
        }
        
        // copy as much as possible into uncommitted buffer
        let available = UNCOMMITTED_BUFFER_CAPACITY - self.uncommitted.len();
        let bytes = buf.len().min(available);
        self.uncommitted.extend_from_slice(&buf[..bytes]);
        
        // uncommitted buffer has been saturated, trigger flush
        if self.uncommitted.len() >= UNCOMMITTED_BUFFER_CAPACITY {
            self.flush()?;
        }
        
        Ok(bytes)
    }
    
    fn flush(&mut self) -> io::Result<()> {
        if self.uncommitted.is_empty() {
            return Ok(());
        }
        
        let result = Self::send_chunk(self.server.queue_handle(), self.request_id, &self.uncommitted);
        
        if result.is_err(){
            self.failed = true;
            return result;
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
        
        if self.flush().is_err() {
            self.disconnect();
            return;
        }
        
        let mut chunk = Self::generate_chunk(b"0\r\n\r\n");
        let mut bytes_sent: c_ulong = 0;
        
        unsafe {
            
            let result = sys::HttpSendResponseEntityBody(
                self.server.queue_handle(),
                self.request_id,
                0,
                1,
                &raw mut chunk,
                &raw mut bytes_sent,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            
            if result != sys::NO_ERROR {
                self.disconnect();
            }
            
        }
    }
    
}
