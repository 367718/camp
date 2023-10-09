mod ffi;
mod session;
mod connection;
mod request;
mod response;

use std::io;

use session::Session;
use connection::Connection;
use request::Request;
use response::Response;

pub fn get(url: &str) -> io::Result<Vec<u8>> {
    let (host, port, path, secure) = extract_params(url)?;
    
    let session = Session::new()?;
    let connection = Connection::new(&session, host, port)?;
    let request = Request::new(&connection, path, secure)?;
    let response = Response::new(request)?;
    
    response.payload()
}

fn extract_params(url: &str) -> io::Result<(&str, u16, &str, bool)> {
    
    fn extract<'a>(url: &'a str, scheme: &str, default_port: u16, secure: bool) -> Option<(&'a str, u16, &'a str, bool)> {
        let base = url.strip_prefix(scheme)?;
        
        let (host_plus_port, path) = match base.find('/') {
            Some(index) => (&base[..index], &base[index..]),
            None => (base, "/"),
        };
        
        let (host, port) = match host_plus_port.split_once(':') {
            Some((host, port)) => (host, port.parse().ok()?),
            None => (host_plus_port, default_port),
        };
        
        if host.is_empty() {
            return None;
        }
        
        Some((host, port, path, secure))
    }
    
    extract(url, "https://", 443, true)
        .or_else(|| extract(url, "http://", 80, false))
        .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "Invalid URL"))
    
}
