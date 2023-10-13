mod ffi;
mod session;
mod connection;
mod payload;

use std::{
    collections::{ hash_map, HashMap },
    io,
};

use session::Session;
use connection::Connection;
use payload::Payload;

const POOL_INITIAL_CAPACITY: usize = 10;

pub struct Client {
    session: Session,
    pool: HashMap<(String, u16), Connection>
}

impl Client {
    
    // -------------------- constructors --------------------
    
    
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            session: Session::new()?,
            pool: HashMap::with_capacity(POOL_INITIAL_CAPACITY),
        })
    }
    
    
    // -------------------- mutators --------------------
    
    
    pub fn get(&mut self, url: &str) -> io::Result<Payload> {
        let (host, port, path, secure) = Self::extract_params(url)?;
        
        let connection = match self.pool.entry((host.to_string(), port)) {
            hash_map::Entry::Occupied(entry) => entry.into_mut(),
            hash_map::Entry::Vacant(entry) => entry.insert(Connection::new(&self.session, host, port)?),
        };
        
        Payload::new(connection, path, secure)
    }
    
    
    // -------------------- helpers --------------------
    
    
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
    
}
