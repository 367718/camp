mod ffi;
mod handle;
mod response;
mod extractor;

use std::{
    io::{ self, Error, ErrorKind },
    os::raw::*,
};

use handle::{ Handle, HandleSource };

pub use response::Response;

const DNS_RESOLUTION_TIMEOUT_AS_MILLIS: c_int = 15_000;
const CONNECTION_TIMEOUT_AS_MILLIS: c_int = 15_000;
const SEND_TIMEOUT_AS_MILLIS: c_int = 15_000;
const RECEIVE_TIMEOUT_AS_MILLIS: c_int = 15_000;

pub struct Client {
    session: Handle,
}

impl Client {
    
    // -------------------- constructors --------------------
    
    
    pub fn new() -> io::Result<Self> {
        let session = ffi::open(env!("CARGO_PKG_NAME"))?;
        
        // set timeout for DNS resolution, connection, send and receive
        ffi::set_timeouts(&session)?;
        
        // set HTTP/2 usage
        ffi::set_option(&session)?;
        
        Ok(Self {
            session,
        })
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn get(&mut self, url: &str) -> io::Result<Response> {
        let (host, port, path, secure) = extractor::get_params(url)
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid URL"))?;
        
        let connection = ffi::connect(&self.session, host, port)?;
        
        let request = ffi::open_request(&connection, path, secure)?;
        ffi::send_request(&request)?;
        
        Response::new(request)
    }
    
}
