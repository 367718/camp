mod ffi;
mod session;
mod connection;
mod payload;
mod extractor;

use std::{
    io::{ self, Error, ErrorKind },
    os::raw::*,
};

use session::Session;
use connection::Connection;

pub use payload::Payload;

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
    
    
    pub fn get(&mut self, url: &str) -> io::Result<Payload> {
        let (host, port, path, secure) = extractor::get_params(url)
            .ok_or(Error::new(ErrorKind::InvalidInput, "Invalid URL"))?;
        
        let connection = Connection::new(&self.session, host, port)?;
        
        Payload::new(&connection, path, secure)
    }
    
}
