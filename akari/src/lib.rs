mod sys;
mod winhttperror;
mod winhttphandle;
mod url;
mod session;
mod connection;
mod request;
mod response;

use std::io;

use winhttperror::last_winhttp_error;
use winhttphandle::WinHttpHandle;
use url::Url;
use session::Session;
use connection::Connection;
use request::Request;

pub use response::Response;

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
    
    pub fn get(&self, resource: &str) -> io::Result<Response<'_>> {
        let url = Url::try_from(resource)?;
        
        let connection = Connection::open(&self.session, &url.host, url.port)?;
        let request = Request::open(connection, &url.path)?;
        
        request.send()
    }
    
}
