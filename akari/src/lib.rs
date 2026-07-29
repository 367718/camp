mod handle;
mod session;
mod connection;
mod request;
mod response;
mod url;

use std::{
    io,
    os::raw::*,
};

use handle::HttpHandle;
use session::Session;
use connection::Connection;
use request::Request;
use url::Url;

pub use response::Response;

const DNS_RESOLUTION_TIMEOUT_AS_MILLIS: c_int = 15_000;
const CONNECTION_TIMEOUT_AS_MILLIS: c_int = 15_000;
const SEND_TIMEOUT_AS_MILLIS: c_int = 15_000;
const RECEIVE_TIMEOUT_AS_MILLIS: c_int = 15_000;

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
        
        self.session.connect(url.host(), url.port())
            .and_then(|connection| connection.send_request(url.path()))
            .and_then(Request::receive_response)
    }
    
}
