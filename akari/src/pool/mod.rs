mod entry;

use std::{
    error::Error,
    time::Duration,
};

use entry::Entry;

pub struct Pool {
    timeout: Duration,
    entries: Vec<Entry>,
}

const CONNECTION_POOL_INITIAL_CAPACITY: usize = 10;

impl Pool {
    
    // ---------- constructors ----------
    
    
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            entries: Vec::with_capacity(CONNECTION_POOL_INITIAL_CAPACITY),
        }
    }
    
    
    // ---------- mutators ----------
    
    
    pub fn get<S: AsRef<str>>(&mut self, url: S) -> Result<Vec<u8>, Box<dyn Error>> {
        let (host, port, path, secure) = Self::extract_params(url.as_ref())
            .ok_or("Invalid URL")?;
        
        let mut entry = self.reuse_entry_or_create_new(host, port, secure)?;
        
        let (body, keep_alive) = entry.send_request(path)?;
        
        if keep_alive {
            self.entries.push(entry);
        }
        
        Ok(body)
    }
    
    fn reuse_entry_or_create_new(&mut self, host: &str, port: u16, secure: bool) -> Result<Entry, Box<dyn Error>> {
        // reused
        if let Some(index) = self.entries.iter().position(|entry| entry.is_connection_already_open(host, port, secure)) {
            let mut entry = self.entries.swap_remove(index);
            
            entry.reopen_connection_if_needed(self.timeout)?;
            
            return Ok(entry);
        }
        
        // new
        Entry::new(host, port, secure, self.timeout)
    }
    
    
    // ---------- helpers ----------
    
    
    fn extract_params(url: &str) -> Option<(&str, u16, &str, bool)> {
        
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
            
            if ! host.is_empty() {
                return Some((host, port, path, secure));
            }
            
            None
        }
        
        extract(url, "https://", 443, true)
            .or_else(|| extract(url, "http://", 80, false))
        
    }
    
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[cfg(test)]
    mod extract_params {
        
        use super::*;
        
        #[test]
        fn valid() {
            // setup
            
            let url = "http://example.com/asd/dsa?query=qwerty";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_some());
            
            let (host, port, path, secure) = output.unwrap();
            
            assert_eq!(host, "example.com");
            assert_eq!(port, 80);
            assert_eq!(path, "/asd/dsa?query=qwerty");
            assert_eq!(secure, false);
        }
        
        #[test]
        fn invalid_scheme() {
            // setup
            
            let url = "file://example.com/asd/dsa?query=qwerty";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn invalid_host() {
            // setup
            
            let url = "http:///asd/dsa?query=qwerty";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn empty() {
            // setup
            
            let url = "";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn scheme_only() {
            // setup
            
            let url = "http://";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn no_scheme() {
            // setup
            
            let url = "example.com/";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_none());
        }
        
        #[test]
        fn root() {
            // setup
            
            let url = "https://example.com/";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_some());
            
            let (host, port, path, secure) = output.unwrap();
            
            assert_eq!(host, "example.com");
            assert_eq!(port, 443);
            assert_eq!(path, "/");
            assert_eq!(secure, true);
        }
        
        #[test]
        fn no_explicit_path() {
            // setup
            
            let url = "https://example.com";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_some());
            
            let (host, port, path, secure) = output.unwrap();
            
            assert_eq!(host, "example.com");
            assert_eq!(port, 443);
            assert_eq!(path, "/");
            assert_eq!(secure, true);
        }
        
        #[test]
        fn ip_and_port() {
            // setup
            
            let url = "http://192.168.10.10:7777/placeholder";
            
            // operation
            
            let output = Pool::extract_params(url);
            
            // control
            
            assert!(output.is_some());
            
            let (host, port, path, secure) = output.unwrap();
            
            assert_eq!(host, "192.168.10.10");
            assert_eq!(port, 7777);
            assert_eq!(path, "/placeholder");
            assert_eq!(secure, false);
        }
        
    }
    
}

