pub fn get_params(url: &str) -> Option<(&str, u16, &str, bool)> {
    extract(url, "https://", 443, true)
        .or_else(|| extract(url, "http://", 80, false))
}

fn extract<'a>(url: &'a str, scheme: &str, default_port: u16, secure: bool) -> Option<(&'a str, u16, &'a str, bool)> {
    let base = url.strip_prefix(scheme)?;
    
    let (host_plus_port, path) = base.find('/')
        .map_or((base, "/"), |index| base.split_at(index));
    
    let (host, port) = match host_plus_port.split_once(':') {
        Some((host, port)) => (host, port.parse().ok()?),
        None => (host_plus_port, default_port),
    };
    
    Some((host, port, path, secure))
}

#[cfg(test)]
mod tests {
    
    use super::*;
    
    #[test]
    fn valid() {
        // setup
        
        let url = "https://example.com/test";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, Some(("example.com", 443, "/test", true)));
    }
    
    #[test]
    fn ip() {
        // setup
        
        let url = "http://192.168.150.10/test";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, Some(("192.168.150.10", 80, "/test", false)));
    }
    
    #[test]
    fn ip_with_port() {
        // setup
        
        let url = "http://192.168.150.10:7777";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, Some(("192.168.150.10", 7777, "/", false)));
    }
    
    #[test]
    fn with_querystring() {
        // setup
        
        let url = "https://example.com/test?placeholder=no&madeup=yes";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, Some(("example.com", 443, "/test?placeholder=no&madeup=yes", true)));
    }
    
    #[test]
    fn with_port() {
        // setup
        
        let url = "http://example.com:8080/test";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, Some(("example.com", 8080, "/test", false)));
    }
    
    #[test]
    fn no_host() {
        // setup
        
        let url = "https://";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, Some(("", 443, "/", true)));
    }
    
    #[test]
    fn invalid_scheme() {
        // setup
        
        let url = "ftp://example.com/test";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, None);
    }
    
    #[test]
    fn invalid_port() {
        // setup
        
        let url = "http://example.com:port/test";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, None);
    }
    
}
