use std::{
    borrow::Cow,
    io::{ Error, ErrorKind },
};

// "secure" flag hardcoded in "response -> receive"
const SUPPORTED_SCHEME: &str = "https://";
const DEFAULT_PORT: u16 = 443;

pub struct Url<'r> {
    pub host: &'r str,
    pub port: u16,
    pub path: Cow<'r, str>,
}

impl<'r> TryFrom<&'r str> for Url<'r> {
    
    type Error = Error;
    
    fn try_from(resource: &'r str) -> Result<Self, Self::Error> {
        // strip scheme
        let base = resource
            .strip_prefix(SUPPORTED_SCHEME)
            .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Unsupported URL scheme"))?;
        
        // extract host_plus_port and raw_path
        let (host_plus_port, raw_path) = match base.find(['/', '?', '#']) {
            Some(index) => base.split_at(index),
            None => (base, "/"),
        };
        
        if host_plus_port.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "Host cannot be empty"));
        }
        
        // fragments (#...) must not be sent to the server
        let path_without_fragment = match raw_path.find('#') {
            Some(idx) => &raw_path[..idx],
            None => raw_path,
        };
        
        // if path is empty, set it to root
        // if path starts with a slash, use it as-is
        // if path does not start with a slash, add it
        let path = if path_without_fragment.is_empty() {
            
            Cow::Borrowed("/")
            
        } else if path_without_fragment.starts_with('/') {
            
            Cow::Borrowed(path_without_fragment)
            
        } else {
            
            let mut path = String::with_capacity(1 + path_without_fragment.len());
            path.push('/');
            path.push_str(path_without_fragment);
            
            Cow::Owned(path)
            
        };
        
        // extract host and port
        let (host, port) = match host_plus_port.split_once(':') {
            Some((host, port)) => (host, port.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid URL port"))?),
            None => (host_plus_port, DEFAULT_PORT),
        };
        
        Ok(Self { host, port, path })
    }
    
}

#[cfg(test)]
mod tests {
    
    // valid
    // ip
    // ip_with_port
    // with_querystring
    // with_port
    // no_host
    // invalid_scheme
    // uppercase_scheme
    // invalid_port
    // empty_port
    // no_slash
    // no_slash_with_querystring
    // no_slash_with_port_and_fragment
    
    use super::*;
    
    #[test]
    fn valid() {
        // setup
        
        let resource = "https://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "example.com");
        assert_eq!(output.port, 443);
        assert_eq!(output.path, "/test");
    }
    
    #[test]
    fn ip() {
        // setup
        
        let resource = "https://192.168.150.10/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "192.168.150.10");
        assert_eq!(output.port, 443);
        assert_eq!(output.path, "/test");
    }
    
    #[test]
    fn ip_with_port() {
        // setup
        
        let resource = "https://192.168.150.10:7777";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "192.168.150.10");
        assert_eq!(output.port, 7777);
        assert_eq!(output.path, "/");
    }
    
    #[test]
    fn with_querystring() {
        // setup
        
        let resource = "https://example.com/test?placeholder=no&madeup=yes";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "example.com");
        assert_eq!(output.port, 443);
        assert_eq!(output.path, "/test?placeholder=no&madeup=yes");
    }
    
    #[test]
    fn with_port() {
        // setup
        
        let resource = "https://example.com:8080/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "example.com");
        assert_eq!(output.port, 8080);
        assert_eq!(output.path, "/test");
    }
    
    #[test]
    fn no_host() {
        // setup
        
        let resource = "https://";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn invalid_scheme() {
        // setup
        
        let resource = "ftp://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn uppercase_scheme() {
        // setup
        
        let resource = "HTTPS://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn invalid_port() {
        // setup
        
        let resource = "https://example.com:port/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn empty_port() {
        // setup
        
        let resource = "https://example.com:/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        assert!(output.is_err());
    }
    
    #[test]
    fn no_slash() {
        // setup
        
        let resource = "https://example.com";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "example.com");
        assert_eq!(output.port, 443);
        assert_eq!(output.path, "/");
    }
    
    #[test]
    fn no_slash_with_querystring() {
        // setup
        
        let resource = "https://example.com?placeholder=no&madeup=yes";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "example.com");
        assert_eq!(output.port, 443);
        assert_eq!(output.path, "/?placeholder=no&madeup=yes");
    }
    
    #[test]
    fn no_slash_with_port_and_fragment() {
        // setup
        
        let resource = "https://example.com:443#frag";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host, "example.com");
        assert_eq!(output.port, 443);
        assert_eq!(output.path, "/");
    }
    
}
