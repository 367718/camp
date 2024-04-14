pub fn get_params(url: &str) -> Option<(&str, u16, &str, bool)> {
    extract(url, "https://", 443, true)
        .or_else(|| extract(url, "http://", 80, false))
}

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
    fn invalid_scheme() {
        // setup
        
        let url = "ftp://example.com/test";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, None);
    }
    
    #[test]
    fn no_host() {
        // setup
        
        let url = "https://";
        
        // operation
        
        let output = get_params(url);
        
        // control
        
        assert_eq!(output, None);
    }
    
}
