use std::io::{ Error, ErrorKind };

// "secure" flag hardcoded in "request" construction
const SUPPORTED_SCHEME: &str = "https://";
const DEFAULT_PORT: u16 = 443;

pub struct Url<'r> {
    host: &'r str,
    port: u16,
    path: &'r str,
}

impl<'r> TryFrom<&'r str> for Url<'r> {
    
    type Error = Error;
    
    fn try_from(resource: &'r str) -> Result<Self, Self::Error> {
        // examples
        // 1 -> "https://example.com/placeholder"
        // 2 -> "https://test.com:8080"
        
        // strip scheme
        // 1 -> "example.com/placeholder"
        // 2 -> "test.com:8080"
        let base = resource.strip_prefix(SUPPORTED_SCHEME)
            .ok_or(Error::new(ErrorKind::InvalidInput, "Unsupported URL scheme"))?;
        
        // extract host_plus_port and path
        // 1 -> "example.com", "/placeholder"
        // 2 -> "test.com:8080", "/"
        let (host_plus_port, path) = base.find('/')
            .map_or((base, "/"), |index| base.split_at(index));
        
        // extract host and port
        // 1 -> "example.com", 443
        // 2 -> "test.com", 8080
        let (host, port) = match host_plus_port.split_once(':') {
            Some((host, port)) => (host, port.parse().map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid URL port"))?),
            None => (host_plus_port, DEFAULT_PORT),
        };
        
        Ok(Self { host, port, path })
    }
    
}

impl Url<'_> {
    
    pub fn host(&self) -> &str {
        self.host
    }
    
    pub fn port(&self) -> u16 {
        self.port
    }
    
    pub fn path(&self) -> &str {
        self.path
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
    
    use super::*;
    
    #[test]
    fn valid() {
        // setup
        
        let resource = "https://example.com/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host(), "example.com");
        assert_eq!(output.port(), 443);
        assert_eq!(output.path(), "/test");
    }
    
    #[test]
    fn ip() {
        // setup
        
        let resource = "https://192.168.150.10/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host(), "192.168.150.10");
        assert_eq!(output.port(), 443);
        assert_eq!(output.path(), "/test");
    }
    
    #[test]
    fn ip_with_port() {
        // setup
        
        let resource = "https://192.168.150.10:7777";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host(), "192.168.150.10");
        assert_eq!(output.port(), 7777);
        assert_eq!(output.path(), "/");
    }
    
    #[test]
    fn with_querystring() {
        // setup
        
        let resource = "https://example.com/test?placeholder=no&madeup=yes";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host(), "example.com");
        assert_eq!(output.port(), 443);
        assert_eq!(output.path(), "/test?placeholder=no&madeup=yes");
    }
    
    #[test]
    fn with_port() {
        // setup
        
        let resource = "https://example.com:8080/test";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host(), "example.com");
        assert_eq!(output.port(), 8080);
        assert_eq!(output.path(), "/test");
    }
    
    #[test]
    fn no_host() {
        // setup
        
        let resource = "https://";
        
        // operation
        
        let output = Url::try_from(resource);
        
        // control
        
        let output = output.unwrap();
        
        assert_eq!(output.host(), "");
        assert_eq!(output.port(), 443);
        assert_eq!(output.path(), "/");
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
    
}
