mod ffi;
mod handle;
mod session;
mod connection;
mod request;
mod response;
mod extractor;

use std::{
    io::{ self, Error, ErrorKind },
    os::raw::*,
};

use handle::Handle;
use session::Session;
use connection::Connection;
use request::Request;

pub use response::Response;

const DNS_RESOLUTION_TIMEOUT_AS_MILLIS: c_int = 15_000;
const CONNECTION_TIMEOUT_AS_MILLIS: c_int = 15_000;
const SEND_TIMEOUT_AS_MILLIS: c_int = 15_000;
const RECEIVE_TIMEOUT_AS_MILLIS: c_int = 15_000;

pub struct Client {
    session: Session,
}

impl Client {
    
    // -------------------- constructors --------------------
    
    
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            session: Session::new()?,
        })
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn get(&mut self, url: &str) -> io::Result<Response> {
        let (host, port, path, secure) = extractor::get_params(url)
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid URL"))?;
        
        let connection = Connection::new(&self.session, host, port)?;
        let request = Request::new(&connection, path, secure)?;
        
        Response::new(request)
    }
    
}
