#[link(name = "winhttp")]
unsafe extern "system" {}

mod winhttphandle;
mod session;
mod connection;
mod response;
mod url;

use std::{
    io,
    os::raw::*,
};

use winhttphandle::WinHttpHandle;
use session::Session;
use connection::Connection;
use url::Url;

pub use response::Response;

const DNS_RESOLUTION_TIMEOUT: c_int = 15_000;
const CONNECTION_TIMEOUT: c_int = 15_000;
const SEND_TIMEOUT: c_int = 15_000;
const RECEIVE_TIMEOUT: c_int = 15_000;

const USER_AGENT: &str = env!("CARGO_PKG_NAME");

pub struct Client {
    session: Session,
}

impl Client {
    
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            session: Session::open(USER_AGENT)?,
        })
    }
    
    pub fn get(&mut self, resource: &str) -> io::Result<Response> {
        let url = Url::try_from(resource)?;
        
        let connection = Connection::open(&self.session, url.host, url.port)?;
        let response = Response::receive(&connection, url.path)?;
        
        Ok(response)
    }
    
}
