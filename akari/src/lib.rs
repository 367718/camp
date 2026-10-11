#[link(name = "winhttp")]
unsafe extern "system" {}

mod winhttperror;
mod winhttphandle;
mod url;
mod session;
mod connection;
mod response;

use std::{
    io,
    os::raw::*,
};

use winhttperror::last_winhttp_error;
use winhttphandle::WinHttpHandle;
use url::Url;
use session::Session;
use connection::Connection;

pub use response::Response;

const USER_AGENT: &str = env!("CARGO_PKG_NAME");

const DNS_RESOLUTION_TIMEOUT: c_int = 15_000;
const CONNECTION_TIMEOUT: c_int = 15_000;
const SEND_TIMEOUT: c_int = 15_000;
const RECEIVE_TIMEOUT: c_int = 15_000;

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
